use serde::Deserialize;

use crate::utils::{
    normalize_city, normalize_email, normalize_phone, normalize_whitespace, password_policy_errors,
};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RegisterForm {
    pub csrf_token: String,
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub city: String,
    pub password: String,
    pub privacy_consent: Option<String>,
}

impl RegisterForm {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if normalize_whitespace(&self.full_name).len() < 3 {
            errors.push("Bitte geben Sie Ihren vollständigen Namen an.".to_string());
        }
        if !self.email.contains('@') {
            errors.push("Bitte geben Sie eine gültige E-Mail-Adresse an.".to_string());
        }
        if normalize_phone(&self.phone_number).len() < 6 {
            errors.push("Bitte geben Sie eine Telefonnummer an.".to_string());
        }
        if normalize_city(&self.city).len() < 2 {
            errors.push("Bitte geben Sie Ihren Wohnort an.".to_string());
        }
        errors.extend(password_policy_errors(&self.password));
        if self.privacy_consent.is_none() {
            errors.push("Bitte bestätigen Sie die Datenschutzhinweise.".to_string());
        }

        errors
    }

    pub fn normalized_email(&self) -> String {
        normalize_email(&self.email)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LoginForm {
    pub csrf_token: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LogoutForm {
    pub csrf_token: String,
}

impl LoginForm {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if !self.email.contains('@') {
            errors.push("Bitte geben Sie eine gültige E-Mail-Adresse ein.".to_string());
        }
        if self.password.trim().is_empty() {
            errors.push("Bitte geben Sie Ihr Passwort ein.".to_string());
        }
        errors
    }

    pub fn normalized_email(&self) -> String {
        normalize_email(&self.email)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BookingForm {
    pub csrf_token: String,
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub city: String,
    pub desired_at: String,
    pub message: String,
    pub password: Option<String>,
    pub privacy_consent: Option<String>,
}

impl BookingForm {
    pub fn validate(&self, requires_password: bool) -> Vec<String> {
        let mut errors = Vec::new();

        if normalize_whitespace(&self.full_name).len() < 3 {
            errors.push("Bitte geben Sie Ihren vollständigen Namen an.".to_string());
        }
        if !self.email.contains('@') {
            errors.push("Bitte geben Sie eine gültige E-Mail-Adresse an.".to_string());
        }
        if normalize_phone(&self.phone_number).len() < 6 {
            errors.push("Bitte geben Sie eine Telefonnummer an.".to_string());
        }
        if normalize_city(&self.city).len() < 2 {
            errors.push("Bitte geben Sie einen Wohnort an.".to_string());
        }
        if self.desired_at.trim().is_empty() {
            errors.push("Bitte wählen Sie einen Wunschtermin aus.".to_string());
        }
        if self.privacy_consent.is_none() {
            errors.push("Bitte bestätigen Sie die Datenschutzhinweise.".to_string());
        }
        if requires_password {
            match &self.password {
                Some(password) if !password.trim().is_empty() => {
                    errors.extend(password_policy_errors(password));
                }
                _ => errors.push(
                    "Für die sichere Terminbuchung benötigen wir ein Passwort für Ihr Kundenkonto."
                        .to_string(),
                ),
            }
        }

        errors
    }

    pub fn normalized_email(&self) -> String {
        normalize_email(&self.email)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AdminAppointmentForm {
    pub csrf_token: String,
    pub desired_at: String,
    pub status: String,
    pub message: String,
    pub total_amount_eur: String,
}

impl AdminAppointmentForm {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.desired_at.trim().is_empty() {
            errors.push("Bitte wählen Sie Datum und Uhrzeit.".to_string());
        }
        if self.status.trim().is_empty() {
            errors.push("Bitte wählen Sie einen Terminstatus.".to_string());
        }
        if self.total_amount_eur.trim().is_empty() {
            errors.push("Bitte geben Sie einen Gesamtbetrag ein.".to_string());
        }
        errors
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AdminPaymentForm {
    pub csrf_token: String,
    pub appointment_id: Option<i64>,
    pub amount_total_eur: String,
    pub amount_paid_eur: String,
    pub payment_date: String,
    pub note: String,
}

impl AdminPaymentForm {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.amount_total_eur.trim().is_empty() {
            errors.push("Bitte geben Sie den Gesamtbetrag ein.".to_string());
        }
        if self.amount_paid_eur.trim().is_empty() {
            errors.push("Bitte geben Sie die eingegangene Zahlung ein.".to_string());
        }
        errors
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AdminNoteForm {
    pub csrf_token: String,
    pub note: String,
}

impl AdminNoteForm {
    pub fn validate(&self) -> Vec<String> {
        if normalize_whitespace(&self.note).len() < 4 {
            vec!["Bitte erfassen Sie eine aussagekräftige interne Notiz.".to_string()]
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppointmentStatusForm {
    pub csrf_token: String,
    pub status: String,
}
