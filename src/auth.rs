use anyhow::Context;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, NaiveDateTime};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};
use sqlx::Row;

use crate::{
    error::{AppError, AppResult},
    models::{AuthenticatedUser, User},
    state::AppState,
    utils::{normalize_email, now_utc},
};

pub const SESSION_COOKIE_NAME: &str = "fbj_session";
pub const CSRF_COOKIE_NAME: &str = "fbj_csrf";

pub fn generate_token() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .context("Passwort konnte nicht gehasht werden")?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed_hash =
        PasswordHash::new(hash).context("Gespeicherter Passwort-Hash ist ungültig")?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub async fn load_current_user(
    state: &AppState,
    jar: &CookieJar,
) -> AppResult<Option<AuthenticatedUser>> {
    let token = match jar.get(SESSION_COOKIE_NAME) {
        Some(cookie) => cookie.value().to_string(),
        None => return Ok(None),
    };

    let token_hash = hash_token(&token);
    let now = now_utc();
    let row = sqlx::query_as::<_, AuthenticatedUser>(
        r#"
        SELECT
            u.id AS user_id,
            c.id AS customer_id,
            u.full_name,
            u.email,
            u.email_verified,
            u.phone_number,
            u.city,
            u.role,
            c.is_active
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        LEFT JOIN customers c ON c.user_id = u.id
        WHERE s.token_hash = ?
          AND s.revoked_at IS NULL
          AND s.expires_at > ?
        "#,
    )
    .bind(&token_hash)
    .bind(now)
    .fetch_optional(&state.pool)
    .await?;

    if row.is_some() {
        sqlx::query("UPDATE sessions SET last_seen_at = ? WHERE token_hash = ?")
            .bind(now)
            .bind(token_hash)
            .execute(&state.pool)
            .await?;
    }

    Ok(row)
}

pub async fn create_session(
    state: &AppState,
    jar: CookieJar,
    user_id: i64,
) -> AppResult<CookieJar> {
    let token = generate_token();
    let token_hash = hash_token(&token);
    let now = now_utc();
    let expires_at = now + Duration::hours(state.config.session_ttl_hours);

    sqlx::query(
        r#"
        INSERT INTO sessions (user_id, token_hash, expires_at, created_at, last_seen_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await?;

    Ok(jar.add(build_cookie(
        SESSION_COOKIE_NAME,
        token,
        state.config.session_cookie_secure,
        true,
        state.config.session_ttl_hours * 3600,
    )))
}

pub async fn destroy_session(state: &AppState, jar: CookieJar) -> AppResult<CookieJar> {
    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
        let token_hash = hash_token(cookie.value());
        sqlx::query("UPDATE sessions SET revoked_at = ? WHERE token_hash = ?")
            .bind(now_utc())
            .bind(token_hash)
            .execute(&state.pool)
            .await?;
    }

    Ok(jar.remove(build_cookie(
        SESSION_COOKIE_NAME,
        String::new(),
        state.config.session_cookie_secure,
        true,
        0,
    )))
}

pub fn ensure_csrf_cookie(state: &AppState, jar: CookieJar) -> (CookieJar, String) {
    if let Some(token) = jar
        .get(CSRF_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
    {
        return (jar, token);
    }

    let token = generate_token();
    let jar = jar.add(build_cookie(
        CSRF_COOKIE_NAME,
        token.clone(),
        state.config.session_cookie_secure,
        false,
        state.config.session_ttl_hours * 3600,
    ));

    (jar, token)
}

pub fn validate_csrf(jar: &CookieJar, form_token: &str) -> AppResult<()> {
    let cookie = jar.get(CSRF_COOKIE_NAME).ok_or_else(|| {
        AppError::BadRequest("CSRF-Schutz konnte nicht validiert werden.".to_string())
    })?;

    if cookie.value() != form_token {
        return Err(AppError::BadRequest(
            "Die Formularsitzung ist abgelaufen. Bitte laden Sie die Seite neu.".to_string(),
        ));
    }

    Ok(())
}

pub fn require_login(user: Option<AuthenticatedUser>) -> AppResult<AuthenticatedUser> {
    user.ok_or(AppError::Unauthorized)
}

pub fn require_admin(user: AuthenticatedUser) -> AppResult<AuthenticatedUser> {
    if user.role == "admin" {
        Ok(user)
    } else {
        Err(AppError::Forbidden)
    }
}

pub fn require_customer(user: AuthenticatedUser) -> AppResult<AuthenticatedUser> {
    if user.role == "customer" {
        Ok(user)
    } else {
        Err(AppError::Forbidden)
    }
}

pub fn build_cookie(
    name: &'static str,
    value: String,
    secure: bool,
    http_only: bool,
    max_age_seconds: i64,
) -> Cookie<'static> {
    Cookie::build((name, value))
        .path("/")
        .same_site(SameSite::Lax)
        .secure(secure)
        .http_only(http_only)
        .max_age(time::Duration::seconds(max_age_seconds))
        .build()
}

pub async fn ensure_email_not_taken(state: &AppState, email: &str) -> AppResult<()> {
    let normalized_email = normalize_email(email);
    let exists = sqlx::query("SELECT id FROM users WHERE email = ?")
        .bind(normalized_email)
        .fetch_optional(&state.pool)
        .await?;

    if exists.is_some() {
        return Err(AppError::BadRequest(
            "Zu dieser E-Mail-Adresse existiert bereits ein Konto. Bitte melden Sie sich an."
                .to_string(),
        ));
    }

    Ok(())
}

pub async fn find_user_by_email(state: &AppState, email: &str) -> AppResult<Option<User>> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(normalize_email(email))
        .fetch_optional(&state.pool)
        .await
        .map_err(Into::into)
}

pub async fn create_email_verification(
    state: &AppState,
    user_id: i64,
    email: &str,
    purpose: &str,
) -> AppResult<String> {
    issue_email_token(
        state,
        user_id,
        email,
        purpose,
        Duration::hours(24),
        TokenKind::Verification,
    )
    .await
}

pub async fn create_password_reset_token(
    state: &AppState,
    user_id: i64,
    email: &str,
) -> AppResult<String> {
    issue_email_token(
        state,
        user_id,
        email,
        "password_reset",
        Duration::hours(2),
        TokenKind::PasswordReset,
    )
    .await
}

pub async fn verification_resend_allowed(state: &AppState, user_id: i64) -> AppResult<bool> {
    let last_created_at = sqlx::query_scalar::<_, NaiveDateTime>(
        r#"
        SELECT created_at
        FROM email_verifications
        WHERE user_id = ?
          AND purpose != 'password_reset'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?;

    let Some(last_created_at) = last_created_at else {
        return Ok(true);
    };

    let next_allowed_at =
        last_created_at + Duration::seconds(state.config.email_resend_cooldown_seconds);
    Ok(now_utc() >= next_allowed_at)
}

pub async fn verify_email_token(state: &AppState, token: &str) -> AppResult<Option<i64>> {
    let token_hash = hash_token(token);
    let now = now_utc();
    let row = sqlx::query(
        r#"
        SELECT id, user_id
        FROM email_verifications
        WHERE token_hash = ?
          AND purpose != 'password_reset'
          AND consumed_at IS NULL
          AND expires_at > ?
        "#,
    )
    .bind(token_hash)
    .bind(now)
    .fetch_optional(&state.pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let verification_id: i64 = row.get("id");
    let user_id: i64 = row.get("user_id");

    let mut tx = state.pool.begin().await?;

    sqlx::query("UPDATE users SET email_verified = 1, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "UPDATE email_verifications SET consumed_at = ? WHERE user_id = ? AND purpose != 'password_reset' AND consumed_at IS NULL",
    )
    .bind(now)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE appointments SET status = 'angefragt', updated_at = ? WHERE customer_id = (SELECT id FROM customers WHERE user_id = ?) AND status = 'wartet_auf_email'",
    )
    .bind(now)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::debug!(
        verification_id,
        user_id,
        "E-Mail-Adresse erfolgreich bestätigt"
    );

    Ok(Some(user_id))
}

pub async fn password_reset_token_is_valid(state: &AppState, token: &str) -> AppResult<bool> {
    let token_hash = hash_token(token);
    let now = now_utc();

    let exists = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM email_verifications
        WHERE token_hash = ?
          AND purpose = 'password_reset'
          AND consumed_at IS NULL
          AND expires_at > ?
        "#,
    )
    .bind(token_hash)
    .bind(now)
    .fetch_one(&state.pool)
    .await?;

    Ok(exists > 0)
}

pub async fn reset_password_with_token(
    state: &AppState,
    token: &str,
    password_hash: &str,
) -> AppResult<Option<i64>> {
    let token_hash = hash_token(token);
    let now = now_utc();
    let row = sqlx::query(
        r#"
        SELECT id, user_id
        FROM email_verifications
        WHERE token_hash = ?
          AND purpose = 'password_reset'
          AND consumed_at IS NULL
          AND expires_at > ?
        "#,
    )
    .bind(token_hash)
    .bind(now)
    .fetch_optional(&state.pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let token_id: i64 = row.get("id");
    let user_id: i64 = row.get("user_id");
    let mut tx = state.pool.begin().await?;

    sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
        .bind(password_hash)
        .bind(now)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "UPDATE email_verifications SET consumed_at = ? WHERE user_id = ? AND purpose = 'password_reset' AND consumed_at IS NULL",
    )
    .bind(now)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE sessions SET revoked_at = ? WHERE user_id = ? AND revoked_at IS NULL")
        .bind(now)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    tracing::debug!(token_id, user_id, "Passwort erfolgreich zurückgesetzt");

    Ok(Some(user_id))
}

enum TokenKind {
    Verification,
    PasswordReset,
}

async fn issue_email_token(
    state: &AppState,
    user_id: i64,
    email: &str,
    purpose: &str,
    ttl: Duration,
    token_kind: TokenKind,
) -> AppResult<String> {
    let token = generate_token();
    let token_hash = hash_token(&token);
    let now = now_utc();
    let expires_at = now + ttl;

    let mut tx = state.pool.begin().await?;

    match token_kind {
        TokenKind::Verification => {
            sqlx::query(
                "UPDATE email_verifications SET consumed_at = ? WHERE user_id = ? AND purpose != 'password_reset' AND consumed_at IS NULL",
            )
            .bind(now)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        }
        TokenKind::PasswordReset => {
            sqlx::query(
                "UPDATE email_verifications SET consumed_at = ? WHERE user_id = ? AND purpose = 'password_reset' AND consumed_at IS NULL",
            )
            .bind(now)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    sqlx::query(
        r#"
        INSERT INTO email_verifications (user_id, token_hash, email, purpose, expires_at, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(normalize_email(email))
    .bind(purpose)
    .bind(expires_at)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(token)
}
