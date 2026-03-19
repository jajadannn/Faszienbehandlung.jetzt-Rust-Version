use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("Nicht authentifiziert.")]
    Unauthorized,
    #[error("Kein Zugriff auf diesen Bereich.")]
    Forbidden,
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    Template(#[from] askama::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    AddressParse(#[from] std::net::AddrParseError),
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Anyhow(_)
            | Self::Io(_)
            | Self::Sqlx(_)
            | Self::SqlxMigrate(_)
            | Self::Template(_)
            | Self::Reqwest(_)
            | Self::AddressParse(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let title = match status {
            StatusCode::BAD_REQUEST => "Eingabe pruefen",
            StatusCode::UNAUTHORIZED => "Anmeldung erforderlich",
            StatusCode::FORBIDDEN => "Zugriff verweigert",
            StatusCode::NOT_FOUND => "Seite nicht gefunden",
            _ => "Interner Fehler",
        };

        let message = match &self {
            Self::BadRequest(message) | Self::NotFound(message) => message.clone(),
            Self::Unauthorized => {
                "Bitte melden Sie sich an, um diesen Bereich zu oeffnen.".to_string()
            }
            Self::Forbidden => {
                "Dieser Bereich ist ausschliesslich fuer berechtigte Personen verfuegbar."
                    .to_string()
            }
            _ => {
                "Beim Verarbeiten der Anfrage ist ein unerwarteter Fehler aufgetreten.".to_string()
            }
        };

        tracing::error!("{}: {}", status, self);

        let html = ErrorTemplate {
            title: title.to_string(),
            message,
            status_code: status.as_u16(),
        }
        .render()
        .unwrap_or_else(|_| "<h1>Interner Fehler</h1>".to_string());

        (status, Html(html)).into_response()
    }
}

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="de">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{ status_code }} | {{ title }}</title>
  <style>
    :root{color-scheme:light;font-family:"Source Sans 3","Segoe UI",sans-serif;}
    body{margin:0;min-height:100vh;display:grid;place-items:center;background:#f7fbfd;color:#173245;}
    main{max-width:42rem;padding:2.5rem;background:white;border-radius:24px;box-shadow:0 24px 60px rgba(17,47,68,.12)}
    h1{margin-top:0;color:#4d93b8}
    a{color:#964279}
  </style>
</head>
<body>
  <main>
    <p>HTTP {{ status_code }}</p>
    <h1>{{ title }}</h1>
    <p>{{ message }}</p>
    <p><a href="/">Zur Startseite</a></p>
  </main>
</body>
</html>"#,
    ext = "html"
)]
struct ErrorTemplate {
    title: String,
    message: String,
    status_code: u16,
}
