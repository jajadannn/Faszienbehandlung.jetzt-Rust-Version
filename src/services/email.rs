use anyhow::Context;
use chrono::NaiveDateTime;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, MultiPart},
    transport::smtp::authentication::Credentials,
};

use crate::{
    config::{AppConfig, SmtpSecurity},
    error::{AppError, AppResult},
    utils::format_datetime,
};

pub struct EmailService {
    sender: MailSender,
    from: Mailbox,
    base_url: String,
    practice_name: String,
    practice_email: String,
    practice_phone: String,
}

struct EmailContent {
    plain: String,
    html: String,
}

enum MailSender {
    LogOnly,
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
}

impl EmailService {
    pub fn from_config(config: &AppConfig) -> AppResult<Self> {
        let sender = match &config.smtp_host {
            Some(host) => MailSender::Smtp(Self::build_smtp_transport(config, host)?),
            None => MailSender::LogOnly,
        };
        let from = config.smtp_from.parse::<Mailbox>().map_err(|error| {
            AppError::Anyhow(anyhow::anyhow!("SMTP_FROM ist ungültig: {error}"))
        })?;

        Ok(Self {
            sender,
            from,
            base_url: config.base_url.clone(),
            practice_name: config.practice_name.clone(),
            practice_email: config.practice_email.clone(),
            practice_phone: config.practice_phone.clone(),
        })
    }

    fn build_smtp_transport(
        config: &AppConfig,
        host: &str,
    ) -> AppResult<AsyncSmtpTransport<Tokio1Executor>> {
        let security = config.smtp_security.resolved_for_port(config.smtp_port);
        let mut builder = match security {
            SmtpSecurity::ImplicitTls => AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                .context("SMTP-Host konnte nicht für implicit TLS initialisiert werden")?,
            SmtpSecurity::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
                .context("SMTP-Host konnte nicht für STARTTLS initialisiert werden")?,
            SmtpSecurity::Plain => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host),
            SmtpSecurity::Auto => unreachable!("AUTO wird vor dem Aufbau aufgelöst"),
        };

        builder = builder.port(config.smtp_port);

        if let (Some(username), Some(password)) = (&config.smtp_username, &config.smtp_password) {
            builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
        }

        tracing::info!(
            smtp_host = host,
            smtp_port = config.smtp_port,
            smtp_security = security.as_str(),
            smtp_auth = config.smtp_username.is_some() && config.smtp_password.is_some(),
            "SMTP-Transport konfiguriert"
        );

        Ok(builder.build())
    }

    pub async fn test_connection(&self) -> AppResult<Option<bool>> {
        match &self.sender {
            MailSender::LogOnly => Ok(None),
            MailSender::Smtp(transport) => {
                transport
                    .test_connection()
                    .await
                    .map(Some)
                    .map_err(|error| {
                        AppError::Anyhow(anyhow::anyhow!(
                            "SMTP-Verbindungstest fehlgeschlagen: {error}"
                        ))
                    })
            }
        }
    }

    pub async fn send_verification_email(
        &self,
        to_email: &str,
        name: &str,
        token: &str,
    ) -> AppResult<()> {
        let verify_url = format!("{}/verify-email?token={}", self.base_url, token);
        let subject = "Bitte bestätigen Sie Ihre E-Mail-Adresse";
        let content = self.verification_content(name, &verify_url);

        self.send_email(to_email, subject, content).await
    }

    pub async fn send_booking_confirmation_email(
        &self,
        to_email: &str,
        name: &str,
        desired_at: &NaiveDateTime,
        email_verified: bool,
    ) -> AppResult<()> {
        let subject = "Ihre Terminanfrage bei faszienbehandlung.jetzt";
        let content = self.booking_confirmation_content(name, desired_at, email_verified);

        self.send_email(to_email, subject, content).await
    }

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        name: &str,
        token: &str,
    ) -> AppResult<()> {
        let reset_url = format!("{}/passwort-zuruecksetzen?token={}", self.base_url, token);
        let subject = "Passwort zurücksetzen";
        let content = self.password_reset_content(name, &reset_url);

        self.send_email(to_email, subject, content).await
    }

    fn verification_content(&self, name: &str, verify_url: &str) -> EmailContent {
        let safe_name = escape_html(name);
        let safe_practice = escape_html(&self.practice_name);
        let safe_verify_url = escape_html(verify_url);

        let plain = format!(
            "Guten Tag {name},\n\nbitte bestätigen Sie Ihre E-Mail-Adresse für Ihr Kundenkonto bei {practice}.\n\nBestätigungslink:\n{verify_url}\n\nDer Link ist 24 Stunden gültig.\nIhre Terminanfrage wird erst nach der Bestätigung der E-Mail-Adresse verbindlich weiterbearbeitet.\n\nFreundliche Grüße\n{practice}",
            name = name.trim(),
            practice = self.practice_name,
            verify_url = verify_url,
        );

        let intro = format!(
            "Guten Tag {name},<br><br>bitte bestätigen Sie Ihre E-Mail-Adresse für Ihr Kundenkonto bei <strong>{practice}</strong>.",
            name = safe_name,
            practice = safe_practice,
        );
        let detail_card = format!(
            "<p style=\"margin:0 0 14px; font-size:16px; line-height:1.75; color:#173245;\">Der Link ist 24 Stunden gültig. Erst danach können wir Ihre Anfrage verbindlich weiterbearbeiten.</p>\
             <p style=\"margin:0; font-size:14px; line-height:1.7; color:#5d7688; word-break:break-all;\">Falls der Button nicht funktioniert, kopieren Sie bitte diesen Link in Ihren Browser:<br>{verify_url}</p>",
            verify_url = safe_verify_url,
        );

        let html = self.render_email_shell(
            "E-Mail-Bestätigung",
            "Bitte bestätigen Sie Ihre E-Mail-Adresse.",
            &intro,
            &detail_card,
            Some(("E-Mail-Adresse bestätigen", verify_url)),
            "Falls Sie diese Registrierung nicht veranlasst haben, können Sie diese Nachricht ignorieren.",
        );

        EmailContent { plain, html }
    }

    fn booking_confirmation_content(
        &self,
        name: &str,
        desired_at: &NaiveDateTime,
        email_verified: bool,
    ) -> EmailContent {
        let desired_at_label = format_datetime(desired_at);
        let safe_name = escape_html(name);
        let safe_practice = escape_html(&self.practice_name);
        let safe_desired_at = escape_html(&desired_at_label);
        let follow_up_plain = if email_verified {
            "Wir melden uns nach interner Prüfung mit einer Rückmeldung zu Ihrem Terminwunsch."
        } else {
            "Bitte bestätigen Sie zuerst Ihre E-Mail-Adresse über den separat gesendeten Verifizierungslink."
        };
        let follow_up_html = escape_html(follow_up_plain);

        let plain = format!(
            "Guten Tag {name},\n\nvielen Dank für Ihre Anfrage bei {practice}.\nGewünschter Termin: {desired_at}\n\n{follow_up}\n\nFreundliche Grüße\n{practice}",
            name = name.trim(),
            practice = self.practice_name,
            desired_at = desired_at_label,
            follow_up = follow_up_plain,
        );

        let intro = format!(
            "Guten Tag {name},<br><br>vielen Dank für Ihre Anfrage bei <strong>{practice}</strong>. Wir haben Ihren Terminwunsch erfasst und intern vorgemerkt.",
            name = safe_name,
            practice = safe_practice,
        );
        let detail_card = format!(
            "<p style=\"margin:0 0 14px; font-size:16px; line-height:1.75; color:#173245;\">\
                <strong>Gewünschter Termin:</strong><br>\
                <span style=\"display:inline-block; margin-top:8px; padding:12px 16px; border-radius:16px; background:#ffffff; color:#4d93b8; font-weight:700;\">{desired_at}</span>\
             </p>\
             <p style=\"margin:0; font-size:15px; line-height:1.8; color:#5d7688;\">{follow_up}</p>",
            desired_at = safe_desired_at,
            follow_up = follow_up_html,
        );

        let html = self.render_email_shell(
            "Terminanfrage eingegangen",
            "Ihre Terminanfrage ist bei uns eingegangen.",
            &intro,
            &detail_card,
            Some(("Zur Webseite", &self.base_url)),
            "Bitte beachten Sie, dass es sich noch nicht um eine fest zugesagte Behandlung handelt.",
        );

        EmailContent { plain, html }
    }

    fn password_reset_content(&self, name: &str, reset_url: &str) -> EmailContent {
        let safe_name = escape_html(name);
        let safe_practice = escape_html(&self.practice_name);
        let safe_reset_url = escape_html(reset_url);

        let plain = format!(
            "Guten Tag {name},\n\nfür Ihr Kundenkonto bei {practice} wurde eine Zurücksetzung des Passworts angefordert.\n\nLink zum Zurücksetzen:\n{reset_url}\n\nDer Link ist aus Sicherheitsgründen nur begrenzt gültig. Falls Sie diese Anfrage nicht selbst ausgelöst haben, können Sie diese E-Mail ignorieren.\n\nFreundliche Grüße\n{practice}",
            name = name.trim(),
            practice = self.practice_name,
            reset_url = reset_url,
        );

        let intro = format!(
            "Guten Tag {name},<br><br>für Ihr Kundenkonto bei <strong>{practice}</strong> wurde eine Zurücksetzung des Passworts angefordert.",
            name = safe_name,
            practice = safe_practice,
        );
        let detail_card = format!(
            "<p style=\"margin:0 0 14px; font-size:16px; line-height:1.75; color:#173245;\">Aus Sicherheitsgründen ist der Link nur begrenzt gültig und kann nur einmal verwendet werden.</p>\
             <p style=\"margin:0; font-size:14px; line-height:1.7; color:#5d7688; word-break:break-all;\">Falls der Button nicht funktioniert, kopieren Sie bitte diesen Link in Ihren Browser:<br>{reset_url}</p>",
            reset_url = safe_reset_url,
        );

        let html = self.render_email_shell(
            "Passwort zurücksetzen",
            "Hier können Sie ein neues Passwort festlegen.",
            &intro,
            &detail_card,
            Some(("Neues Passwort festlegen", reset_url)),
            "Falls Sie diese Anfrage nicht selbst ausgelöst haben, können Sie diese Nachricht ignorieren.",
        );

        EmailContent { plain, html }
    }

    fn render_email_shell(
        &self,
        eyebrow: &str,
        title: &str,
        intro_html: &str,
        detail_card_html: &str,
        cta: Option<(&str, &str)>,
        footer_note: &str,
    ) -> String {
        let safe_practice = escape_html(&self.practice_name);
        let safe_practice_email = escape_html(&self.practice_email);
        let safe_practice_phone = escape_html(&self.practice_phone);
        let safe_eyebrow = escape_html(eyebrow);
        let safe_title = escape_html(title);
        let safe_footer_note = escape_html(footer_note);

        let cta_html = cta
            .map(|(label, url)| {
                format!(
                    "<table role=\"presentation\" cellspacing=\"0\" cellpadding=\"0\" border=\"0\" style=\"margin:28px auto 0;\">\
                        <tr>\
                            <td align=\"center\" bgcolor=\"#964279\" style=\"border-radius:999px;\">\
                                <a href=\"{url}\" style=\"display:inline-block; padding:16px 28px; color:#ffffff; text-decoration:none; font-size:16px; font-weight:700;\">{label}</a>\
                            </td>\
                        </tr>\
                    </table>",
                    url = escape_html(url),
                    label = escape_html(label),
                )
            })
            .unwrap_or_default();

        format!(
            "<!doctype html>\
            <html lang=\"de\">\
            <head>\
                <meta charset=\"utf-8\">\
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\
                <title>{title}</title>\
            </head>\
            <body style=\"margin:0; padding:0; background:#f7fbfd; color:#173245;\">\
                <table role=\"presentation\" width=\"100%\" cellspacing=\"0\" cellpadding=\"0\" border=\"0\" style=\"background:#f7fbfd;\">\
                    <tr>\
                        <td align=\"center\" style=\"padding:32px 16px;\">\
                            <table role=\"presentation\" width=\"100%\" cellspacing=\"0\" cellpadding=\"0\" border=\"0\" style=\"max-width:640px; background:#ffffff; border:1px solid #dcecf3; border-radius:28px; overflow:hidden; box-shadow:0 24px 60px rgba(23,50,69,0.08);\">\
                                <tr>\
                                    <td style=\"padding:0;\">\
                                        <div style=\"height:12px; background:linear-gradient(90deg, #4d93b8 0%, #70AECD 55%, #a8cfe0 100%);\"></div>\
                                        <div style=\"padding:30px 32px 26px; background:linear-gradient(180deg, #dff0f7 0%, #ffffff 100%);\">\
                                            <div style=\"display:inline-block; margin-bottom:18px; padding:8px 14px; border-radius:999px; background:#ffffff; color:#964279; font-size:12px; font-weight:700; letter-spacing:0.14em; text-transform:uppercase;\">{eyebrow}</div>\
                                            <div style=\"font-size:14px; line-height:1.6; color:#4d93b8; font-weight:700; margin-bottom:10px;\">faszienbehandlung.jetzt</div>\
                                            <h1 style=\"margin:0; color:#173245; font-family:Georgia, 'Times New Roman', serif; font-size:42px; line-height:1.06; font-weight:500;\">{title}</h1>\
                                        </div>\
                                    </td>\
                                </tr>\
                                <tr>\
                                    <td style=\"padding:32px; font-family:'Segoe UI', Arial, sans-serif;\">\
                                        <div style=\"font-size:17px; line-height:1.8; color:#173245;\">{intro}</div>\
                                        <div style=\"margin-top:24px; padding:22px 24px; border-radius:22px; background:#dff0f7; border:1px solid #b7d7e5;\">{detail_card}</div>\
                                        {cta_html}\
                                        <p style=\"margin:28px 0 0; font-size:14px; line-height:1.8; color:#5d7688;\">{footer_note}</p>\
                                    </td>\
                                </tr>\
                                <tr>\
                                    <td style=\"padding:24px 32px 32px; border-top:1px solid #e6f0f5; background:#ffffff; font-family:'Segoe UI', Arial, sans-serif;\">\
                                        <div style=\"font-size:14px; line-height:1.8; color:#173245; font-weight:700; margin-bottom:6px;\">{practice}</div>\
                                        <div style=\"font-size:14px; line-height:1.8; color:#5d7688;\">\
                                            Kontakt: <a href=\"mailto:{practice_email}\" style=\"color:#964279; text-decoration:none;\">{practice_email}</a> &middot; <a href=\"tel:{practice_phone_tel}\" style=\"color:#964279; text-decoration:none;\">{practice_phone}</a><br>\
                                            Diese Nachricht wurde automatisch für Ihre Anfrage auf <a href=\"{base_url}\" style=\"color:#4d93b8; text-decoration:none;\">faszienbehandlung.jetzt</a> erstellt.\
                                        </div>\
                                    </td>\
                                </tr>\
                            </table>\
                        </td>\
                    </tr>\
                </table>\
            </body>\
            </html>",
            title = safe_title,
            eyebrow = safe_eyebrow,
            intro = intro_html,
            detail_card = detail_card_html,
            cta_html = cta_html,
            footer_note = safe_footer_note,
            practice = safe_practice,
            practice_email = safe_practice_email,
            practice_phone = safe_practice_phone,
            practice_phone_tel = escape_html(&self.practice_phone.replace(' ', "")),
            base_url = escape_html(&self.base_url),
        )
    }

    async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        content: EmailContent,
    ) -> AppResult<()> {
        match &self.sender {
            MailSender::LogOnly => {
                tracing::info!(
                    "Log-only E-Mail an {} | {} | {}",
                    to_email,
                    subject,
                    content.plain
                );
                Ok(())
            }
            MailSender::Smtp(transport) => {
                let email = Message::builder()
                    .from(self.from.clone())
                    .to(to_email.parse::<Mailbox>().map_err(|error| {
                        AppError::BadRequest(format!("Empfängeradresse ist ungültig: {error}"))
                    })?)
                    .subject(subject)
                    .multipart(MultiPart::alternative_plain_html(
                        content.plain,
                        content.html,
                    ))
                    .context("E-Mail-Nachricht konnte nicht erstellt werden")?;

                transport.send(email).await.map_err(|error| {
                    tracing::error!("E-Mail-Versand fehlgeschlagen: {error}");
                    AppError::Anyhow(anyhow::anyhow!(
                        "E-Mail konnte nicht gesendet werden: {error}"
                    ))
                })?;

                Ok(())
            }
        }
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
