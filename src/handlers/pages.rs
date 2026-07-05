use askama::Template;
use axum::{extract::State, response::Response};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    config::AppConfig,
    content,
    error::AppResult,
    state::AppState,
    utils::format_cents,
    views::{FaqItemView, PageShell, ServiceCardView},
};

use super::{build_shell, render};

pub async fn home(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/",
        "Faszienbehandlung in ruhiger Praxisatmosphäre | faszienbehandlung.jetzt",
        "Professionelle Faszienbehandlung mit klarer Nutzerführung, transparenter Preisstruktur und sicherem Online-Buchungssystem auf www.faszienbehandlung.jetzt.",
    )
    .await?;

    let template = HomeTemplate {
        structured_data: structured_data_json(&state.config),
        shell,
    };

    render(jar, &template)
}

fn structured_data_json(config: &AppConfig) -> String {
    let data = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "HealthAndBeautyBusiness",
        "name": config.practice_name,
        "url": config.base_url,
        "telephone": config.practice_phone,
        "email": config.practice_email,
        "address": {
            "@type": "PostalAddress",
            "streetAddress": config.practice_address_line_1,
            "addressLocality": config.practice_address_line_2,
            "addressCountry": "DE"
        },
        "openingHours": [
            config.opening_hours_weekdays.replace(" Uhr", ""),
            config.opening_hours_saturday.replace(" Uhr", "")
        ],
        "priceRange": format!("ab {}", format_cents(config.booking_base_price_cents)),
        "image": format!("{}/static/Logo-256.v2.png", config.base_url)
    });

    serde_json::to_string(&data)
        .unwrap_or_default()
        .replace('<', "\\u003c")
}

pub async fn about(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/praxis",
        "Praxis für Faszienbehandlung | Über uns",
        "Einblick in Haltung, Ablauf und Praxisumfeld der Faszienbehandlung auf www.faszienbehandlung.jetzt.",
    )
    .await?;

    let template = BasicTemplate {
        shell,
        path_key: "about".to_string(),
    };
    render(jar, &template)
}

pub async fn fascia_info(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/faszienbehandlung",
        "Was ist Faszienbehandlung? | sachlich erklärt",
        "Verständliche und SEO-freundliche Erklärseite zur Faszienbehandlung mit seriöser Einordnung ohne Heilversprechen.",
    )
    .await?;

    let template = BasicTemplate {
        shell,
        path_key: "fascia".to_string(),
    };
    render(jar, &template)
}

pub async fn services(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let title = format!(
        "Leistungen & Preise | Faszienbehandlung in {}",
        state.config.practice_region_label
    );
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/leistungen-preise",
        &title,
        "Übersicht zu Leistungen, Termindauern und transparenten Preisen für www.faszienbehandlung.jetzt.",
    )
    .await?;

    let template = ServicesTemplate {
        services: content::service_cards(&shell.practice),
        shell,
    };
    render(jar, &template)
}

pub async fn faq(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/faq",
        "FAQ zur Faszienbehandlung und Terminbuchung",
        "Antworten zu Behandlung, Terminbuchung, Datenschutz, Kundenkonto und Zahlungsstatus bei www.faszienbehandlung.jetzt.",
    )
    .await?;

    let template = FaqTemplate {
        faqs: content::faq_items(&shell.practice),
        shell,
    };
    render(jar, &template)
}

pub async fn contact(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/kontakt",
        "Kontakt | Praxis für Faszienbehandlung",
        "Kontaktseite mit Praxisdaten, Erreichbarkeit und direkter Verlinkung zur Terminbuchung auf www.faszienbehandlung.jetzt.",
    )
    .await?;

    let template = BasicTemplate {
        shell,
        path_key: "contact".to_string(),
    };
    render(jar, &template)
}

pub async fn imprint(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/impressum",
        "Impressum | faszienbehandlung.jetzt",
        "Impressum mit Platzhaltern für die spätere fachliche Prüfung vor dem Livegang.",
    )
    .await?;

    let template = BasicTemplate {
        shell,
        path_key: "imprint".to_string(),
    };
    render(jar, &template)
}

pub async fn privacy(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/datenschutz",
        "Datenschutz | faszienbehandlung.jetzt",
        "DSGVO-Grundgerüst für Hosting, Server-Logs, Kontakt, Terminbuchung, Registrierung, Login, E-Mail-Verifizierung und Session-Cookies.",
    )
    .await?;

    let template = BasicTemplate {
        shell,
        path_key: "privacy".to_string(),
    };
    render(jar, &template)
}

#[derive(Template)]
#[template(path = "pages/home.html")]
struct HomeTemplate {
    shell: PageShell,
    structured_data: String,
}

#[derive(Template)]
#[template(path = "pages/basic.html")]
struct BasicTemplate {
    shell: PageShell,
    path_key: String,
}

#[derive(Template)]
#[template(path = "pages/services.html")]
struct ServicesTemplate {
    shell: PageShell,
    services: Vec<ServiceCardView>,
}

#[derive(Template)]
#[template(path = "pages/faq.html")]
struct FaqTemplate {
    shell: PageShell,
    faqs: Vec<FaqItemView>,
}
