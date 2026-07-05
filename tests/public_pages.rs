mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use faszienbehandlung_jetzt::handlers::seo_routes::PUBLIC_PATHS;
use tower::ServiceExt;

async fn get_body(path: &str) -> (StatusCode, String, String) {
    let (app, _config) = common::test_app().await;
    let response = app
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .map(|value| value.to_str().unwrap_or_default().to_string())
        .unwrap_or_default();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    (
        status,
        content_type,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

#[tokio::test]
async fn all_public_pages_render() {
    for path in PUBLIC_PATHS {
        let (status, content_type, body) = get_body(path).await;
        assert_eq!(status, StatusCode::OK, "Pfad {path} liefert kein 200");
        assert!(
            content_type.starts_with("text/html"),
            "Pfad {path} liefert kein HTML ({content_type})"
        );
        assert!(!body.is_empty(), "Pfad {path} liefert leeren Body");
    }
}

#[tokio::test]
async fn home_page_content_is_consistent() {
    let (status, _, body) = get_body("/").await;
    assert_eq!(status, StatusCode::OK);

    assert!(body.contains("Robert Gantke"), "Founder-Name fehlt");
    assert!(
        !body.contains("Frank Gantke"),
        "Falscher Founder-Name im HTML"
    );
    assert!(
        body.contains("application/ld+json"),
        "JSON-LD strukturierte Daten fehlen"
    );
    assert!(
        !body.contains("1 Jahr G"),
        "Hartkodierte Paket-Gültigkeit noch vorhanden"
    );
    assert!(
        body.contains("Monate G"),
        "Config-getriebene Paket-Gültigkeit fehlt"
    );
}

#[tokio::test]
async fn home_duration_comes_from_config() {
    let (app, config) = common::test_app().await;
    let response = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&bytes).into_owned();

    let expected = format!("ca. {} Minuten", config.appointment_duration_minutes);
    assert!(
        body.contains(&expected),
        "Behandlungsdauer aus der Konfiguration fehlt: {expected}"
    );
}

#[tokio::test]
async fn legal_pages_are_structured() {
    let (_, _, imprint) = get_body("/impressum").await;
    assert!(imprint.contains("5 DDG"), "Impressum ohne §-5-DDG-Struktur");
    assert!(
        imprint.contains("[BITTE ERG"),
        "Impressum ohne sichtbare Platzhalter-Markierung"
    );

    let (_, _, privacy) = get_body("/datenschutz").await;
    assert!(
        privacy.contains("Verantwortlicher"),
        "Datenschutz ohne Verantwortlichen"
    );
    assert!(
        privacy.contains("Nominatim"),
        "Datenschutz erwähnt Nominatim-Geokodierung nicht"
    );
    assert!(
        privacy.contains("Argon2"),
        "Datenschutz erwähnt Passwort-Hashing nicht"
    );
}

#[tokio::test]
async fn unknown_path_returns_404_with_security_headers() {
    let (app, _config) = common::test_app().await;
    let response = app
        .oneshot(Request::get("/unbekannt").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(
        response.headers().contains_key("content-security-policy"),
        "CSP-Header fehlt auf Fehlerseite"
    );
}
