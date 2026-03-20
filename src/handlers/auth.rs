use askama::Template;
use axum::{
    Form,
    extract::{Query, State},
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::{
    auth as auth_helpers,
    error::{AppError, AppResult},
    forms::{ForgotPasswordForm, LoginForm, LogoutForm, RegisterForm, ResetPasswordForm},
    models::User,
    state::AppState,
    utils::{normalize_city, normalize_phone, now_utc},
    views::{FlashMessage, PageShell},
};

use super::{build_shell, redirect, render};

pub async fn show_register(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/registrierung",
        "Registrierung | Kundenkonto für Faszienbehandlung",
        "Sichere Registrierung mit Passwort-Hashing, Datenschutz-Zustimmung und Ortsvalidierung für www.faszienbehandlung.jetzt.",
    )
    .await?;

    let template = RegisterTemplate {
        shell,
        form: RegisterForm::default(),
        errors: Vec::new(),
        flash: None,
    };

    render(jar, &template)
}

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<RegisterForm>,
) -> AppResult<Response> {
    auth_helpers::validate_csrf(&jar, &form.csrf_token)?;

    let mut errors = form.validate();
    if errors.is_empty() {
        if let Err(error) = auth_helpers::ensure_email_not_taken(&state, &form.email).await {
            match error {
                AppError::BadRequest(message) => errors.push(message),
                other => return Err(other),
            }
        }
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
    }

    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/registrierung",
        "Registrierung | Kundenkonto für Faszienbehandlung",
        "Sichere Registrierung mit Passwort-Hashing, Datenschutz-Zustimmung und Ortsvalidierung für www.faszienbehandlung.jetzt.",
    )
    .await?;

    if !errors.is_empty() {
        let template = RegisterTemplate {
            shell,
            form,
            errors,
            flash: None,
        };
        return render(jar, &template);
    }

    let now = now_utc();
    let password_hash = auth_helpers::hash_password(&form.password)?;
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

    sqlx::query(
        "INSERT INTO customers (user_id, is_active, created_at, updated_at) VALUES (?, 1, ?, ?)",
    )
    .bind(user_id)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let token =
        auth_helpers::create_email_verification(&state, user_id, &form.email, "registration")
            .await?;
    state
        .email_service
        .send_verification_email(&form.normalized_email(), &form.full_name, &token)
        .await?;

    let template = RegisterTemplate {
        shell,
        form: RegisterForm::default(),
        errors: Vec::new(),
        flash: Some(FlashMessage {
            kind: "success".to_string(),
            title: "Registrierung abgeschlossen".to_string(),
            text: "Ihr Kundenkonto wurde angelegt. Bitte bestätigen Sie jetzt Ihre E-Mail-Adresse, damit Sie Termine buchen können.".to_string(),
        }),
    };

    render(jar, &template)
}

pub async fn show_login(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    render_login_page(&state, jar, LoginForm::default(), Vec::new(), None).await
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> AppResult<Response> {
    auth_helpers::validate_csrf(&jar, &form.csrf_token)?;

    let mut errors = form.validate();
    let normalized_email = form.normalized_email();

    let user = if errors.is_empty() {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(&normalized_email)
            .fetch_optional(&state.pool)
            .await?
    } else {
        None
    };

    if let Some(user) = &user {
        if !auth_helpers::verify_password(&form.password, &user.password_hash)? {
            errors.push("Die E-Mail-Adresse oder das Passwort stimmt nicht.".to_string());
        }
    } else if errors.is_empty() {
        errors.push("Die E-Mail-Adresse oder das Passwort stimmt nicht.".to_string());
    }

    if !errors.is_empty() {
        return render_login_page(
            &state,
            jar,
            form.clone(),
            errors,
            None,
        )
        .await;
    }

    let user = user.expect("checked above");
    if user.role == "customer" {
        let active = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM customers WHERE user_id = ? AND is_active = 1",
        )
        .bind(user.id)
        .fetch_one(&state.pool)
        .await?;
        if active == 0 {
            return Err(AppError::BadRequest(
                "Dieses Kundenkonto ist derzeit nicht aktiv. Bitte kontaktieren Sie die Praxis."
                    .to_string(),
            ));
        }
    }

    let jar = auth_helpers::create_session(&state, jar, user.id).await?;
    let target = if user.role == "admin" {
        "/admin"
    } else {
        "/konto"
    };

    Ok(redirect(jar, target))
}

pub async fn show_forgot_password(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<Response> {
    render_forgot_password_page(
        &state,
        jar,
        ForgotPasswordForm::default(),
        Vec::new(),
        None,
    )
    .await
}

pub async fn request_password_reset(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<ForgotPasswordForm>,
) -> AppResult<Response> {
    auth_helpers::validate_csrf(&jar, &form.csrf_token)?;

    let errors = form.validate();
    if !errors.is_empty() {
        return render_forgot_password_page(&state, jar, form, errors, None).await;
    }

    if let Some(user) = auth_helpers::find_user_by_email(&state, &form.email).await? {
        let token =
            auth_helpers::create_password_reset_token(&state, user.id, &user.email).await?;
        state
            .email_service
            .send_password_reset_email(&user.email, &user.full_name, &token)
            .await?;
    }

    render_forgot_password_page(
        &state,
        jar,
        ForgotPasswordForm::default(),
        Vec::new(),
        Some(FlashMessage {
            kind: "success".to_string(),
            title: "Falls ein Konto existiert".to_string(),
            text: "Wenn zu dieser E-Mail-Adresse ein Kundenkonto hinterlegt ist, haben wir soeben eine E-Mail zum Zurücksetzen des Passworts versendet.".to_string(),
        }),
    )
    .await
}

pub async fn show_reset_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<ResetPasswordQuery>,
) -> AppResult<Response> {
    let token = query.token.unwrap_or_default();
    let token_valid = if token.is_empty() {
        false
    } else {
        auth_helpers::password_reset_token_is_valid(&state, &token).await?
    };

    let flash = if token_valid {
        None
    } else {
        Some(FlashMessage {
            kind: "warning".to_string(),
            title: "Link prüfen".to_string(),
            text: "Der Zurücksetzungslink ist ungültig, unvollständig oder bereits abgelaufen. Fordern Sie bei Bedarf einfach einen neuen Link an.".to_string(),
        })
    };

    render_reset_password_page(
        &state,
        jar,
        ResetPasswordForm {
            token,
            ..ResetPasswordForm::default()
        },
        Vec::new(),
        flash,
        token_valid,
    )
    .await
}

pub async fn reset_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<ResetPasswordForm>,
) -> AppResult<Response> {
    auth_helpers::validate_csrf(&jar, &form.csrf_token)?;

    let mut errors = form.validate();
    let token_valid = if form.token.trim().is_empty() {
        false
    } else {
        auth_helpers::password_reset_token_is_valid(&state, &form.token).await?
    };

    if !token_valid {
        errors.push("Der Zurücksetzungslink ist ungültig oder abgelaufen.".to_string());
    }

    if !errors.is_empty() {
        return render_reset_password_page(
            &state,
            jar,
            ResetPasswordForm {
                token: form.token.clone(),
                ..ResetPasswordForm::default()
            },
            errors,
            None,
            token_valid,
        )
        .await;
    }

    let password_hash = auth_helpers::hash_password(&form.password)?;
    if auth_helpers::reset_password_with_token(&state, &form.token, &password_hash)
        .await?
        .is_none()
    {
        return render_reset_password_page(
            &state,
            jar,
            ResetPasswordForm {
                token: form.token,
                ..ResetPasswordForm::default()
            },
            vec!["Der Zurücksetzungslink ist ungültig oder abgelaufen.".to_string()],
            None,
            false,
        )
        .await;
    }

    render_reset_password_page(
        &state,
        jar,
        ResetPasswordForm::default(),
        Vec::new(),
        Some(FlashMessage {
            kind: "success".to_string(),
            title: "Passwort aktualisiert".to_string(),
            text: "Ihr Passwort wurde erfolgreich geändert. Sie können sich jetzt mit dem neuen Passwort anmelden.".to_string(),
        }),
        false,
    )
    .await
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LogoutForm>,
) -> AppResult<Response> {
    auth_helpers::validate_csrf(&jar, &form.csrf_token)?;
    let jar = auth_helpers::destroy_session(&state, jar).await?;
    Ok(redirect(jar, "/"))
}

pub async fn verify_email(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<VerifyEmailQuery>,
) -> AppResult<Response> {
    let success = match query.token {
        Some(token) => auth_helpers::verify_email_token(&state, &token)
            .await?
            .is_some(),
        None => false,
    };

    render_verify_page(
        &state,
        jar,
        success,
        None,
    )
    .await
}

async fn render_login_page(
    state: &AppState,
    jar: CookieJar,
    form: LoginForm,
    errors: Vec<String>,
    flash: Option<FlashMessage>,
) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        state,
        jar,
        "/anmeldung",
        "Anmeldung | Kundenkonto & Admin-Login",
        "Sicherer Login für Kundenkonto und Admin-Bereich auf www.faszienbehandlung.jetzt.",
    )
    .await?;

    let template = LoginTemplate {
        shell,
        form,
        errors,
        flash,
    };

    render(jar, &template)
}

async fn render_verify_page(
    state: &AppState,
    jar: CookieJar,
    success: bool,
    flash: Option<FlashMessage>,
) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        state,
        jar,
        "/verify-email",
        "E-Mail bestätigen | faszienbehandlung.jetzt",
        "Bestätigung der E-Mail-Adresse für das Kundenkonto und die sichere Terminbuchung.",
    )
    .await?;

    let template = VerifyTemplate {
        shell,
        success,
        flash,
    };
    render(jar, &template)
}

async fn render_forgot_password_page(
    state: &AppState,
    jar: CookieJar,
    form: ForgotPasswordForm,
    errors: Vec<String>,
    flash: Option<FlashMessage>,
) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        state,
        jar,
        "/passwort-vergessen",
        "Passwort zurücksetzen | Kundenkonto",
        "Sichere Zurücksetzung des Passworts per E-Mail-Link für das Kundenkonto auf www.faszienbehandlung.jetzt.",
    )
    .await?;

    let template = ForgotPasswordTemplate {
        shell,
        form,
        errors,
        flash,
    };
    render(jar, &template)
}

async fn render_reset_password_page(
    state: &AppState,
    jar: CookieJar,
    form: ResetPasswordForm,
    errors: Vec<String>,
    flash: Option<FlashMessage>,
    token_valid: bool,
) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        state,
        jar,
        "/passwort-zuruecksetzen",
        "Neues Passwort festlegen | Kundenkonto",
        "Neues Passwort sicher festlegen, nachdem der Zurücksetzungslink aus der E-Mail geöffnet wurde.",
    )
    .await?;

    let template = ResetPasswordTemplate {
        shell,
        form,
        errors,
        flash,
        token_valid,
    };
    render(jar, &template)
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailQuery {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordQuery {
    pub token: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/register.html")]
struct RegisterTemplate {
    shell: PageShell,
    form: RegisterForm,
    errors: Vec<String>,
    flash: Option<FlashMessage>,
}

#[derive(Template)]
#[template(path = "pages/login.html")]
struct LoginTemplate {
    shell: PageShell,
    form: LoginForm,
    errors: Vec<String>,
    flash: Option<FlashMessage>,
}

#[derive(Template)]
#[template(path = "pages/verify.html")]
struct VerifyTemplate {
    shell: PageShell,
    success: bool,
    flash: Option<FlashMessage>,
}

#[derive(Template)]
#[template(path = "pages/forgot_password.html")]
struct ForgotPasswordTemplate {
    shell: PageShell,
    form: ForgotPasswordForm,
    errors: Vec<String>,
    flash: Option<FlashMessage>,
}

#[derive(Template)]
#[template(path = "pages/reset_password.html")]
struct ResetPasswordTemplate {
    shell: PageShell,
    form: ResetPasswordForm,
    errors: Vec<String>,
    flash: Option<FlashMessage>,
    token_valid: bool,
}
