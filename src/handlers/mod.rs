pub mod account;
pub mod admin;
pub mod auth;
pub mod booking;
pub mod pages;
pub mod seo_routes;

use askama::Template;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie::CookieJar;
use chrono::{Datelike, Utc};
use serde::Serialize;
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{
    auth as auth_helpers,
    error::{AppError, AppResult},
    middleware::security_headers,
    models::AuthenticatedUser,
    seo::SeoMeta,
    state::AppState,
    utils::format_cents,
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
        .route(
            "/passwort-vergessen",
            get(auth::show_forgot_password).post(auth::request_password_reset),
        )
        .route(
            "/passwort-zuruecksetzen",
            get(auth::show_reset_password).post(auth::reset_password),
        )
        .route("/logout", post(auth::logout))
        .route("/konto", get(account::dashboard))
        .route(
            "/konto/verify-email/resend",
            post(account::resend_verification),
        )
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
        .route("/robots.txt", get(seo_routes::robots_txt))
        .route("/sitemap.xml", get(seo_routes::sitemap_xml))
        .route("/__dev/reload", get(dev_reload_state))
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
    let config = &state.config;

    let package_savings_cents = (config.booking_base_price_cents
        - config.booking_package_session_price_cents)
        * config.booking_package_session_count;
    let package_savings_label = if package_savings_cents > 0 {
        format!("Spare {} gesamt", euro_symbol_label(package_savings_cents))
    } else {
        "Preisvorteil für wiederkehrende Termine individuell konfigurierbar".to_string()
    };

    let shell = PageShell {
        meta: SeoMeta::new(config, path, title, description),
        current_user: current_user.as_ref().map(Into::into),
        csrf_token,
        year: Utc::now().year(),
        auto_reload_enabled: config.auto_reload_enabled,
        auto_reload_interval_ms: config.auto_reload_interval_ms,
        auto_reload_endpoint: "/__dev/reload".to_string(),
        server_instance_id: state.server_instance_id.as_ref().clone(),
        practice: PracticeView {
            name: config.practice_name.clone(),
            practitioner_name: config.practitioner_name.clone(),
            email: config.practice_email.clone(),
            phone: config.practice_phone.clone(),
            address_line_1: config.practice_address_line_1.clone(),
            address_line_2: config.practice_address_line_2.clone(),
            region_label: config.practice_region_label.clone(),
            house_call_area: config.practice_house_call_area.clone(),
            opening_hours_weekdays: config.opening_hours_weekdays.clone(),
            opening_hours_saturday: config.opening_hours_saturday.clone(),
            opening_hours_summary: format!(
                "{} · {}",
                config.opening_hours_weekdays, config.opening_hours_saturday
            ),
            appointment_duration_short: format!("{} Min.", config.appointment_duration_minutes),
            appointment_duration_verbose: format!(
                "ca. {} Minuten",
                config.appointment_duration_minutes
            ),
            single_session_price_label: euro_symbol_label(config.booking_base_price_cents),
            single_session_price_input: plain_euro_value(config.booking_base_price_cents),
            package_session_price_label: euro_symbol_label(
                config.booking_package_session_price_cents,
            ),
            package_session_price_input: plain_euro_value(
                config.booking_package_session_price_cents,
            ),
            package_card_label: format!(
                "{}er-Karte · pro Sitzung",
                config.booking_package_session_count
            ),
            package_validity_label: format!(
                "Übertragbar · {} Monate gültig",
                config.booking_package_validity_months
            ),
            package_validity_short: format!(
                "{} Monate Gültigkeit",
                config.booking_package_validity_months
            ),
            package_savings_label,
            house_call_fee_label: euro_symbol_label(config.house_call_fee_cents),
            house_call_fee_input: plain_euro_value(config.house_call_fee_cents),
            maps_query: maps_query(
                &config.practice_address_line_1,
                &config.practice_address_line_2,
            ),
        },
    };

    Ok((jar, shell, current_user))
}

fn euro_symbol_label(cents: i64) -> String {
    format_cents(cents).replace(" EUR", " €")
}

fn plain_euro_value(cents: i64) -> String {
    format_cents(cents).replace(" EUR", "")
}

fn maps_query(address_line_1: &str, address_line_2: &str) -> String {
    format!("{address_line_1}, {address_line_2}").replace(' ', "+")
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

#[derive(Serialize)]
struct ReloadState {
    instance_id: String,
}

async fn dev_reload_state(State(state): State<AppState>) -> Response {
    if !state.config.auto_reload_enabled {
        return StatusCode::NOT_FOUND.into_response();
    }

    Json(ReloadState {
        instance_id: state.server_instance_id.as_ref().clone(),
    })
    .into_response()
}
