use askama::Template;
use axum::{Form, extract::State, response::Response};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    auth as auth_helpers,
    error::{AppError, AppResult},
    forms::BookingForm,
    models::AuthenticatedUser,
    state::AppState,
    utils::{format_datetime, normalize_city, normalize_phone, now_utc, parse_datetime_local},
    views::{FlashMessage, PageShell},
};

use super::{build_shell, render};

pub async fn show_booking(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, current_user) = build_shell(
        &state,
        jar,
        "/terminbuchung",
        "Terminbuchung | Faszienbehandlung online anfragen",
        "Prominente, DSGVO-bewusste Terminbuchung mit Kundenkonto, E-Mail-Verifizierung und gepruefter Wohnortvalidierung.",
    )
    .await?;

    let form = prefill_form(current_user.as_ref());
    let flash = current_user.as_ref().and_then(|user| {
        if !user.email_verified && user.role == "customer" {
            Some(FlashMessage {
                kind: "warning".to_string(),
                title: "E-Mail-Bestaetigung ausstehend".to_string(),
                text: "Bitte bestaetigen Sie zuerst Ihre E-Mail-Adresse, bevor Sie eine verbindliche Terminanfrage senden.".to_string(),
            })
        } else {
            None
        }
    });

    let template = BookingTemplate {
        shell,
        form,
        errors: Vec::new(),
        flash,
        requires_password: current_user.is_none(),
        logged_in: current_user.is_some(),
    };

    render(jar, &template)
}

pub async fn submit_booking(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<BookingForm>,
) -> AppResult<Response> {
    auth_helpers::validate_csrf(&jar, &form.csrf_token)?;
    let current_user = auth_helpers::load_current_user(&state, &jar).await?;
    let requires_password = current_user.is_none();

    let mut errors = form.validate(requires_password);

    if let Some(user) = &current_user {
        if user.role != "customer" {
            errors.push("Nur Kundenkonten koennen Online-Termine buchen.".to_string());
        }
        if form.normalized_email() != user.email {
            errors.push(
                "Bitte verwenden Sie fuer die Buchung dieselbe E-Mail-Adresse wie in Ihrem Kundenkonto."
                    .to_string(),
            );
        }
        if !user.email_verified {
            errors.push(
                "Bitte bestaetigen Sie zunaechst Ihre E-Mail-Adresse, bevor Sie einen Termin anfragen."
                    .to_string(),
            );
        }
    }

    let desired_at = if errors.is_empty() {
        Some(parse_datetime_local(&form.desired_at)?)
    } else {
        None
    };

    if errors.is_empty() {
        if let Err(error) = state
            .location_service
            .validate_city(&state.pool, &form.city)
            .await
        {
            match error {
                AppError::BadRequest(message) => errors.push(message),
                other => return Err(other),
            }
        }
        if current_user.is_none() {
            if let Err(error) = auth_helpers::ensure_email_not_taken(&state, &form.email).await {
                match error {
                    AppError::BadRequest(message) => errors.push(message),
                    other => return Err(other),
                }
            }
        }
    }

    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/terminbuchung",
        "Terminbuchung | Faszienbehandlung online anfragen",
        "Prominente, DSGVO-bewusste Terminbuchung mit Kundenkonto, E-Mail-Verifizierung und gepruefter Wohnortvalidierung.",
    )
    .await?;

    if !errors.is_empty() {
        let template = BookingTemplate {
            shell,
            form,
            errors,
            flash: None,
            requires_password,
            logged_in: current_user.is_some(),
        };
        return render(jar, &template);
    }

    let desired_at = desired_at.expect("validated above");
    let now = now_utc();
    let base_price_cents = state.config.booking_base_price_cents;

    let (created_account, needs_verification, customer_name, customer_email) = if let Some(user) =
        current_user
    {
        let customer_user = auth_helpers::require_customer(user)?;
        let customer_id = customer_user.customer_id.ok_or_else(|| {
            AppError::BadRequest(
                "Zu diesem Konto konnte kein Kundenprofil geladen werden.".to_string(),
            )
        })?;

        let mut tx = state.pool.begin().await?;
        sqlx::query(
            "UPDATE users SET full_name = ?, phone_number = ?, city = ?, updated_at = ? WHERE id = ?",
        )
        .bind(form.full_name.trim())
        .bind(normalize_phone(&form.phone_number))
        .bind(normalize_city(&form.city))
        .bind(now)
        .bind(customer_user.user_id)
        .execute(&mut *tx)
        .await?;

        let appointment_result = sqlx::query(
            r#"
            INSERT INTO appointments
                (customer_id, desired_at, status, message, total_amount_cents, created_at, updated_at)
            VALUES (?, ?, 'angefragt', ?, ?, ?, ?)
            "#,
        )
        .bind(customer_id)
        .bind(desired_at)
        .bind(empty_to_none(&form.message))
        .bind(base_price_cents)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        let appointment_id = appointment_result.last_insert_rowid();
        sqlx::query(
            r#"
            INSERT INTO payments
                (customer_id, appointment_id, amount_total_cents, amount_paid_cents, amount_open_cents, status, note, created_at, updated_at)
            VALUES (?, ?, ?, 0, ?, 'offen', ?, ?, ?)
            "#,
        )
        .bind(customer_id)
        .bind(appointment_id)
        .bind(base_price_cents)
        .bind(base_price_cents)
        .bind("Online-Anfrage ueber Kundenkonto.")
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        state
            .email_service
            .send_booking_confirmation_email(
                &customer_user.email,
                &form.full_name,
                &desired_at,
                customer_user.email_verified,
            )
            .await?;

        (false, false, customer_user.full_name, customer_user.email)
    } else {
        let password = form.password.clone().unwrap_or_default();
        let password_hash = auth_helpers::hash_password(&password)?;
        let mut tx = state.pool.begin().await?;

        let user_result = sqlx::query(
            r#"
            INSERT INTO users
                (full_name, email, email_verified, phone_number, city, password_hash, role, created_at, updated_at)
            VALUES (?, ?, 0, ?, ?, ?, 'customer', ?, ?)
            "#,
        )
        .bind(form.full_name.trim())
        .bind(form.normalized_email())
        .bind(normalize_phone(&form.phone_number))
        .bind(normalize_city(&form.city))
        .bind(password_hash)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        let user_id = user_result.last_insert_rowid();

        let customer_result = sqlx::query(
            "INSERT INTO customers (user_id, is_active, created_at, updated_at) VALUES (?, 1, ?, ?)",
        )
        .bind(user_id)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        let customer_id = customer_result.last_insert_rowid();

        let appointment_result = sqlx::query(
            r#"
            INSERT INTO appointments
                (customer_id, desired_at, status, message, total_amount_cents, created_at, updated_at)
            VALUES (?, ?, 'wartet_auf_email', ?, ?, ?, ?)
            "#,
        )
        .bind(customer_id)
        .bind(desired_at)
        .bind(empty_to_none(&form.message))
        .bind(base_price_cents)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        let appointment_id = appointment_result.last_insert_rowid();

        sqlx::query(
            r#"
            INSERT INTO payments
                (customer_id, appointment_id, amount_total_cents, amount_paid_cents, amount_open_cents, status, note, created_at, updated_at)
            VALUES (?, ?, ?, 0, ?, 'offen', ?, ?, ?)
            "#,
        )
        .bind(customer_id)
        .bind(appointment_id)
        .bind(base_price_cents)
        .bind(base_price_cents)
        .bind("Online-Anfrage mit neuer Registrierung.")
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let token =
            auth_helpers::create_email_verification(&state, user_id, &form.email, "booking")
                .await?;
        state
            .email_service
            .send_verification_email(&form.normalized_email(), &form.full_name, &token)
            .await?;
        state
            .email_service
            .send_booking_confirmation_email(
                &form.normalized_email(),
                &form.full_name,
                &desired_at,
                false,
            )
            .await?;

        (true, true, form.full_name.clone(), form.normalized_email())
    };

    let template = BookingSuccessTemplate {
        shell,
        created_account,
        needs_verification,
        desired_at_label: format_datetime(&desired_at),
        customer_name,
        customer_email,
    };
    render(jar, &template)
}

fn prefill_form(current_user: Option<&AuthenticatedUser>) -> BookingForm {
    match current_user {
        Some(user) => BookingForm {
            full_name: user.full_name.clone(),
            email: user.email.clone(),
            phone_number: user.phone_number.clone(),
            city: user.city.clone(),
            ..BookingForm::default()
        },
        None => BookingForm::default(),
    }
}

fn empty_to_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[derive(Template)]
#[template(path = "pages/booking.html")]
struct BookingTemplate {
    shell: PageShell,
    form: BookingForm,
    errors: Vec<String>,
    flash: Option<FlashMessage>,
    requires_password: bool,
    logged_in: bool,
}

#[derive(Template)]
#[template(path = "pages/booking_success.html")]
struct BookingSuccessTemplate {
    shell: PageShell,
    created_account: bool,
    needs_verification: bool,
    desired_at_label: String,
    customer_name: String,
    customer_email: String,
}
