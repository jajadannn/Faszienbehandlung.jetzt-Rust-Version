use anyhow::Context;
use chrono::NaiveDateTime;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox,
    transport::smtp::authentication::Credentials,
};

use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    utils::format_datetime,
};

pub struct EmailService {
    sender: MailSender,
    from: String,
    base_url: String,
    practice_name: String,
}

enum MailSender {
    LogOnly,
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
}

impl EmailService {
    pub fn from_config(config: &AppConfig) -> AppResult<Self> {
        let sender = match &config.smtp_host {
            Some(host) => {
                let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                    .context("SMTP-Host konnte nicht initialisiert werden")?;
                builder = builder.port(config.smtp_port);

                if let (Some(username), Some(password)) =
                    (&config.smtp_username, &config.smtp_password)
                {
                    builder =
                        builder.credentials(Credentials::new(username.clone(), password.clone()));
                }

                MailSender::Smtp(builder.build())
            }
            None => MailSender::LogOnly,
        };

        Ok(Self {
            sender,
            from: config.smtp_from.clone(),
            base_url: config.base_url.clone(),
            practice_name: config.practice_name.clone(),
        })
    }

    pub async fn send_verification_email(
        &self,
        to_email: &str,
        name: &str,
        token: &str,
    ) -> AppResult<()> {
        let verify_url = format!("{}/verify-email?token={}", self.base_url, token);
        let subject = "Bitte bestätigen Sie Ihre E-Mail-Adresse";
        let body = format!(
            "Guten Tag {name},\n\nbitte bestätigen Sie Ihre E-Mail-Adresse für Ihr Kundenkonto bei {practice}:\n{verify_url}\n\nDer Link ist 24 Stunden gültig.\n\nHinweis: Ihre Terminanfrage wird erst nach der Bestätigung Ihrer E-Mail-Adresse verbindlich weiterbearbeitet.\n\nFreundliche Grüße\n{practice}",
            practice = self.practice_name
        );

        self.send_email(to_email, subject, &body).await
    }

    pub async fn send_booking_confirmation_email(
        &self,
        to_email: &str,
        name: &str,
        desired_at: &NaiveDateTime,
        email_verified: bool,
    ) -> AppResult<()> {
        let subject = "Ihre Terminanfrage bei faszienbehandlung.jetzt";
        let follow_up = if email_verified {
            "Wir melden uns nach interner Prüfung mit einer Rückmeldung zu Ihrem Terminwunsch."
        } else {
            "Bitte bestätigen Sie zuerst Ihre E-Mail-Adresse über den gesendeten Verifizierungslink."
        };
        let body = format!(
            "Guten Tag {name},\n\nvielen Dank für Ihre Anfrage bei {practice}.\nGewünschter Termin: {desired_at}\n\n{follow_up}\n\nFreundliche Grüße\n{practice}",
            practice = self.practice_name,
            desired_at = format_datetime(desired_at),
        );

        self.send_email(to_email, subject, &body).await
    }

    async fn send_email(&self, to_email: &str, subject: &str, body: &str) -> AppResult<()> {
        match &self.sender {
            MailSender::LogOnly => {
                tracing::info!("Log-only E-Mail an {} | {} | {}", to_email, subject, body);
                Ok(())
            }
            MailSender::Smtp(transport) => {
                let email = Message::builder()
                    .from(self.from.parse::<Mailbox>().map_err(|error| {
                        AppError::BadRequest(format!("SMTP_FROM ist ungültig: {error}"))
                    })?)
                    .to(to_email.parse::<Mailbox>().map_err(|error| {
                        AppError::BadRequest(format!("Empfängeradresse ist ungültig: {error}"))
                    })?)
                    .subject(subject)
                    .body(body.to_string())
                    .context("E-Mail-Nachricht konnte nicht erstellt werden")?;

                transport.send(email).await.map_err(|error| {
                    AppError::BadRequest(format!("E-Mail konnte nicht gesendet werden: {error}"))
                })?;

                Ok(())
            }
        }
    }
}
