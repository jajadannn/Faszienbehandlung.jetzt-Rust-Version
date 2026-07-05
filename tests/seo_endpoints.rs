mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use faszienbehandlung_jetzt::handlers::seo_routes::PUBLIC_PATHS;
use tower::ServiceExt;

#[tokio::test]
async fn robots_txt_references_sitemap() {
    let (app, config) = common::test_app().await;
    let response = app
        .oneshot(Request::get("/robots.txt").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response.headers()["content-type"].to_str().unwrap();
    assert!(content_type.starts_with("text/plain"));

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&bytes).into_owned();

    assert!(body.contains(&format!("Sitemap: {}/sitemap.xml", config.base_url)));
    assert!(body.contains("Disallow: /admin"));
    assert!(body.contains("Disallow: /konto"));
}

#[tokio::test]
async fn sitemap_lists_all_public_paths() {
    let (app, config) = common::test_app().await;
    let response = app
        .oneshot(Request::get("/sitemap.xml").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response.headers()["content-type"].to_str().unwrap();
    assert!(content_type.starts_with("application/xml"));

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&bytes).into_owned();

    for path in PUBLIC_PATHS {
        let loc = if *path == "/" {
            format!("<loc>{}</loc>", config.base_url)
        } else {
            format!("<loc>{}{}</loc>", config.base_url, path)
        };
        assert!(body.contains(&loc), "Sitemap-Eintrag fehlt: {loc}");
    }
}
