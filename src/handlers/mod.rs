pub mod account;
pub mod admin;
pub mod auth;
pub mod booking;
pub mod pages;

use askama::Template;
use axum::{
    Router,
    extract::State,
    middleware,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie::CookieJar;
use chrono::{Datelike, Utc};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{
    auth as auth_helpers,
    error::{AppError, AppResult},
    middleware::security_headers,
    models::AuthenticatedUser,
    seo::SeoMeta,
    state::AppState,
    views::{PageShell, PracticeView},
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(pages::home))
        .route("/praxis", get(pages::about))
        .route("/faszienbehandlung", get(pages::fascia_info))
        .route("/leistungen-preise", get(pages::services))
        .route("/faq", get(pages::faq))
        .route("/kontakt", get(pages::contact))
        .route(
            "/terminbuchung",
            get(booking::show_booking).post(booking::submit_booking),
        )
        .route(
            "/registrierung",
            get(auth::show_register).post(auth::register),
        )
        .route("/anmeldung", get(auth::show_login).post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/konto", get(account::dashboard))
        .route("/admin", get(admin::dashboard))
        .route("/admin/kunden/:customer_id", get(admin::customer_detail))
        .route("/admin/kunden/:customer_id/notizen", post(admin::add_note))
        .route(
            "/admin/kunden/:customer_id/zahlungen",
            post(admin::add_payment),
        )
        .route(
            "/admin/kunden/:customer_id/termine",
            post(admin::add_appointment),
        )
        .route(
            "/admin/kunden/:customer_id/termine/:appointment_id/status",
            post(admin::update_appointment_status),
        )
        .route("/impressum", get(pages::imprint))
        .route("/datenschutz", get(pages::privacy))
        .route("/verify-email", get(auth::verify_email))
        .nest_service("/static", ServeDir::new("static"))
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_headers::apply_security_headers,
        ))
        .with_state(state)
}

pub async fn build_shell(
    state: &AppState,
    jar: CookieJar,
    path: &str,
    title: &str,
    description: &str,
) -> AppResult<(CookieJar, PageShell, Option<AuthenticatedUser>)> {
    let current_user = auth_helpers::load_current_user(state, &jar).await?;
    let (jar, csrf_token) = auth_helpers::ensure_csrf_cookie(state, jar);

    let shell = PageShell {
        meta: SeoMeta::new(&state.config, path, title, description),
        current_user: current_user.as_ref().map(Into::into),
        csrf_token,
        year: Utc::now().year(),
        practice: PracticeView {
            name: state.config.practice_name.clone(),
            email: state.config.practice_email.clone(),
            phone: state.config.practice_phone.clone(),
            address_line_1: state.config.practice_address_line_1.clone(),
            address_line_2: state.config.practice_address_line_2.clone(),
        },
    };

    Ok((jar, shell, current_user))
}

pub fn render<T: Template>(jar: CookieJar, template: &T) -> AppResult<Response> {
    Ok((jar, Html(template.render()?)).into_response())
}

async fn not_found(State(_state): State<AppState>) -> impl IntoResponse {
    AppError::NotFound("Die angeforderte Seite existiert nicht oder wurde verschoben.".to_string())
}

pub fn redirect(jar: CookieJar, path: &str) -> Response {
    (jar, Redirect::to(path)).into_response()
}
