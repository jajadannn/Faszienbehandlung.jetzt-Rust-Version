use askama::Template;
use axum::{extract::State, response::Response};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    content,
    error::AppResult,
    state::AppState,
    views::{FaqItemView, InfoCard, PageShell, ServiceCardView, StatCard},
};

use super::{build_shell, render};

pub async fn home(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/",
        "Faszienbehandlung in ruhiger Praxisatmosphaere | faszienbehandlung.jetzt",
        "Professionelle Faszienbehandlung mit klarer Nutzerfuehrung, transparenter Preisstruktur und sicherem Online-Buchungssystem auf www.faszienbehandlung.jetzt.",
    )
    .await?;

    let template = HomeTemplate {
        shell,
        stats: content::home_stats(),
        complaints: content::complaint_cards(),
        benefits: content::benefit_cards(),
        trust: content::trust_cards(),
        services: content::service_cards(),
        faqs: content::faq_items().into_iter().take(4).collect(),
    };

    render(jar, &template)
}

pub async fn about(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/praxis",
        "Praxis fuer Faszienbehandlung | Ueber uns",
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
        "Was ist Faszienbehandlung? | sachlich erklaert",
        "Verstaendliche und SEO-freundliche Erklaerseite zur Faszienbehandlung mit serioeser Einordnung ohne Heilversprechen.",
    )
    .await?;

    let template = BasicTemplate {
        shell,
        path_key: "fascia".to_string(),
    };
    render(jar, &template)
}

pub async fn services(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/leistungen-preise",
        "Leistungen & Preise | Faszienbehandlung in Berlin",
        "Uebersicht zu Leistungen, Termindauern und transparenten Preisen fuer www.faszienbehandlung.jetzt.",
    )
    .await?;

    let template = ServicesTemplate {
        shell,
        services: content::service_cards(),
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
        shell,
        faqs: content::faq_items(),
    };
    render(jar, &template)
}

pub async fn contact(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/kontakt",
        "Kontakt | Praxis fuer Faszienbehandlung",
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
        "Impressum mit Platzhaltern fuer die spaetere fachliche Pruefung vor dem Livegang.",
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
        "DSGVO-Grundgeruest fuer Hosting, Server-Logs, Kontakt, Terminbuchung, Registrierung, Login, E-Mail-Verifizierung und Session-Cookies.",
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
    stats: Vec<StatCard>,
    complaints: Vec<InfoCard>,
    benefits: Vec<InfoCard>,
    trust: Vec<InfoCard>,
    services: Vec<ServiceCardView>,
    faqs: Vec<FaqItemView>,
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
