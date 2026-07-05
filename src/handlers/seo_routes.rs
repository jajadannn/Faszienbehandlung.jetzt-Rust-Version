use axum::{
    extract::State,
    http::header,
    response::{IntoResponse, Response},
};

use crate::state::AppState;

pub const PUBLIC_PATHS: &[&str] = &[
    "/",
    "/praxis",
    "/faszienbehandlung",
    "/leistungen-preise",
    "/faq",
    "/kontakt",
    "/terminbuchung",
    "/impressum",
    "/datenschutz",
];

pub async fn robots_txt(State(state): State<AppState>) -> Response {
    let body = format!(
        "User-agent: *\n\
         Disallow: /konto\n\
         Disallow: /admin\n\
         Disallow: /anmeldung\n\
         Disallow: /registrierung\n\
         Disallow: /passwort-vergessen\n\
         Disallow: /passwort-zuruecksetzen\n\
         Disallow: /verify-email\n\
         Disallow: /__dev/\n\
         \n\
         Sitemap: {}/sitemap.xml\n",
        state.config.base_url
    );

    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}

pub async fn sitemap_xml(State(state): State<AppState>) -> Response {
    let base_url = &state.config.base_url;
    let mut body = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );

    for path in PUBLIC_PATHS {
        let loc = if *path == "/" {
            base_url.clone()
        } else {
            format!("{base_url}{path}")
        };
        body.push_str(&format!("  <url><loc>{loc}</loc></url>\n"));
    }

    body.push_str("</urlset>\n");

    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        body,
    )
        .into_response()
}
