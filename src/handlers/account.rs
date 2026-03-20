use askama::Template;
use axum::{Form, extract::State, response::Response};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    auth as auth_helpers,
    error::{AppError, AppResult},
    forms::CsrfForm,
    models::{AppointmentOverviewRow, AuthenticatedUser, CustomerSummaryRow, Payment},
    state::AppState,
    utils::{
        appointment_status_label, format_cents, format_datetime, now_utc, payment_status_label,
    },
    views::{AppointmentView, CustomerProfileView, FlashMessage, PageShell, PaymentView},
};

use super::{build_shell, render};

pub async fn dashboard(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let current_user = load_customer_user(&state, &jar).await?;
    render_dashboard_page(&state, jar, current_user, None).await
}

pub async fn resend_verification(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<CsrfForm>,
) -> AppResult<Response> {
    auth_helpers::validate_csrf(&jar, &form.csrf_token)?;
    let current_user = load_customer_user(&state, &jar).await?;

    let flash = if current_user.email_verified {
        FlashMessage {
            kind: "warning".to_string(),
            title: "Bereits bestätigt".to_string(),
            text: "Die hinterlegte E-Mail-Adresse ist bereits bestätigt. Ein neuer Bestätigungslink ist nicht mehr erforderlich.".to_string(),
        }
    } else if !auth_helpers::verification_resend_allowed(&state, current_user.user_id).await? {
        FlashMessage {
            kind: "warning".to_string(),
            title: "Bitte kurz warten".to_string(),
            text: format!(
                "Ein neuer Bestätigungslink kann höchstens alle {} angefordert werden.",
                format_cooldown(state.config.email_resend_cooldown_seconds)
            ),
        }
    } else {
        let token = auth_helpers::create_email_verification(
            &state,
            current_user.user_id,
            &current_user.email,
            "resend_verification",
        )
        .await?;
        state
            .email_service
            .send_verification_email(&current_user.email, &current_user.full_name, &token)
            .await?;

        FlashMessage {
            kind: "success".to_string(),
            title: "Bestätigungslink gesendet".to_string(),
            text: "Wir haben einen neuen Bestätigungslink an die in Ihrem Kundenkonto hinterlegte E-Mail-Adresse gesendet.".to_string(),
        }
    };

    render_dashboard_page(&state, jar, current_user, Some(flash)).await
}

async fn load_customer_user(state: &AppState, jar: &CookieJar) -> AppResult<AuthenticatedUser> {
    auth_helpers::require_customer(auth_helpers::require_login(
        auth_helpers::load_current_user(state, jar).await?,
    )?)
}

async fn render_dashboard_page(
    state: &AppState,
    jar: CookieJar,
    current_user: AuthenticatedUser,
    flash: Option<FlashMessage>,
) -> AppResult<Response> {
    let customer_id = current_user.customer_id.ok_or_else(|| {
        AppError::BadRequest("Zu diesem Konto existiert kein Kundenprofil.".to_string())
    })?;

    let profile = sqlx::query_as::<_, CustomerSummaryRow>(
        r#"
        SELECT
            c.id AS customer_id,
            u.id AS user_id,
            u.full_name,
            u.email,
            u.phone_number,
            u.city,
            u.email_verified,
            c.is_active,
            (SELECT COUNT(*) FROM appointments a WHERE a.customer_id = c.id) AS appointment_count,
            (SELECT MAX(desired_at) FROM appointments a WHERE a.customer_id = c.id AND a.desired_at <= CURRENT_TIMESTAMP) AS last_appointment_at,
            (SELECT MIN(desired_at) FROM appointments a WHERE a.customer_id = c.id AND a.desired_at > CURRENT_TIMESTAMP AND a.status != 'storniert') AS next_appointment_at,
            COALESCE((SELECT SUM(amount_total_cents) FROM payments p WHERE p.customer_id = c.id), 0) AS total_value_cents,
            COALESCE((SELECT SUM(amount_paid_cents) FROM payments p WHERE p.customer_id = c.id), 0) AS total_paid_cents,
            COALESCE((SELECT SUM(amount_open_cents) FROM payments p WHERE p.customer_id = c.id), 0) AS total_open_cents
        FROM customers c
        JOIN users u ON u.id = c.user_id
        WHERE c.id = ? AND u.id = ?
        "#,
    )
    .bind(customer_id)
    .bind(current_user.user_id)
    .fetch_one(&state.pool)
    .await?;

    let appointments = sqlx::query_as::<_, AppointmentOverviewRow>(
        r#"
        SELECT
            a.id,
            a.desired_at,
            a.status,
            a.message,
            a.total_amount_cents,
            COALESCE(p.amount_paid_cents, 0) AS amount_paid_cents,
            COALESCE(p.amount_open_cents, a.total_amount_cents) AS amount_open_cents
        FROM appointments a
        LEFT JOIN payments p ON p.appointment_id = a.id
        WHERE a.customer_id = ?
        ORDER BY a.desired_at DESC
        "#,
    )
    .bind(customer_id)
    .fetch_all(&state.pool)
    .await?;

    let payments = sqlx::query_as::<_, Payment>(
        "SELECT * FROM payments WHERE customer_id = ? ORDER BY created_at DESC",
    )
    .bind(customer_id)
    .fetch_all(&state.pool)
    .await?;

    let appointment_views: Vec<AppointmentView> = appointments
        .into_iter()
        .map(|appointment| AppointmentView {
            id: appointment.id,
            desired_at: appointment.desired_at,
            desired_at_label: format_datetime(&appointment.desired_at),
            status: appointment.status.clone(),
            status_label: appointment_status_label(&appointment.status).to_string(),
            message: appointment
                .message
                .unwrap_or_else(|| "Keine zusätzliche Nachricht".to_string()),
            total_amount_label: format_cents(appointment.total_amount_cents),
            paid_amount_label: format_cents(appointment.amount_paid_cents),
            open_amount_label: format_cents(appointment.amount_open_cents),
        })
        .collect();

    let now = now_utc();
    let future_appointments = appointment_views
        .iter()
        .filter(|appointment| appointment.desired_at > now && appointment.status != "storniert")
        .cloned()
        .collect();
    let past_appointments = appointment_views
        .iter()
        .filter(|appointment| {
            appointment.desired_at <= now || appointment.status == "abgeschlossen"
        })
        .cloned()
        .collect();

    let payment_views = payments
        .into_iter()
        .map(|payment| PaymentView {
            id: payment.id,
            amount_total_label: format_cents(payment.amount_total_cents),
            amount_paid_label: format_cents(payment.amount_paid_cents),
            amount_open_label: format_cents(payment.amount_open_cents),
            status_label: payment_status_label(&payment.status).to_string(),
            payment_date_label: payment
                .payment_date
                .map(|value| format_datetime(&value))
                .unwrap_or_else(|| "Noch kein Zahlungseingang".to_string()),
            note: payment
                .note
                .unwrap_or_else(|| "Kein Hinweis hinterlegt".to_string()),
        })
        .collect();

    let (jar, shell, _) = build_shell(
        state,
        jar,
        "/konto",
        "Kundenkonto | Termine, Zahlungen und Stammdaten",
        "Geschützter Kundenbereich mit persönlichen Stammdaten, Terminstatus, Zahlungsverlauf und Übersicht offener Beträge.",
    )
    .await?;

    let template = AccountTemplate {
        shell,
        flash,
        profile: CustomerProfileView {
            customer_id: profile.customer_id,
            full_name: profile.full_name,
            email: profile.email,
            phone_number: profile.phone_number,
            city: profile.city,
            email_verified: profile.email_verified,
            email_verified_label: if profile.email_verified {
                "Ja".to_string()
            } else {
                "Nein".to_string()
            },
            status_label: if profile.is_active {
                "Aktiv".to_string()
            } else {
                "Inaktiv".to_string()
            },
            appointment_count: profile.appointment_count.to_string(),
            last_appointment_label: profile
                .last_appointment_at
                .map(|value| format_datetime(&value))
                .unwrap_or_else(|| "Noch kein abgeschlossener Termin".to_string()),
            next_appointment_label: profile
                .next_appointment_at
                .map(|value| format_datetime(&value))
                .unwrap_or_else(|| "Kein geplanter Termin".to_string()),
            total_paid_label: format_cents(profile.total_paid_cents),
            total_open_label: format_cents(profile.total_open_cents),
            total_value_label: format_cents(profile.total_value_cents),
        },
        future_appointments,
        past_appointments,
        payments: payment_views,
    };

    render(jar, &template)
}

fn format_cooldown(seconds: i64) -> String {
    match seconds {
        s if s >= 3600 && s % 3600 == 0 => {
            let hours = s / 3600;
            if hours == 1 {
                "1 Stunde".to_string()
            } else {
                format!("{hours} Stunden")
            }
        }
        s if s >= 60 && s % 60 == 0 => {
            let minutes = s / 60;
            if minutes == 1 {
                "1 Minute".to_string()
            } else {
                format!("{minutes} Minuten")
            }
        }
        1 => "1 Sekunde".to_string(),
        s => format!("{s} Sekunden"),
    }
}

#[derive(Template)]
#[template(path = "pages/account.html")]
struct AccountTemplate {
    shell: PageShell,
    flash: Option<FlashMessage>,
    profile: CustomerProfileView,
    future_appointments: Vec<AppointmentView>,
    past_appointments: Vec<AppointmentView>,
    payments: Vec<PaymentView>,
}
