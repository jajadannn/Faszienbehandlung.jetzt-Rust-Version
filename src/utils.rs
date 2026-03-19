use anyhow::Context;
use chrono::{NaiveDateTime, Utc};

use crate::error::{AppError, AppResult};

pub fn now_utc() -> NaiveDateTime {
    Utc::now().naive_utc()
}

pub fn format_datetime(datetime: &NaiveDateTime) -> String {
    datetime.format("%d.%m.%Y, %H:%M Uhr").to_string()
}

pub fn format_date(datetime: &NaiveDateTime) -> String {
    datetime.format("%d.%m.%Y").to_string()
}

pub fn parse_datetime_local(input: &str) -> AppResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M")
        .with_context(|| format!("Ungültiges Datum/Zeit-Format: {}", input))
        .map_err(AppError::from)
}

pub fn format_cents(cents: i64) -> String {
    let euros = cents / 100;
    let cents_part = (cents % 100).abs();
    let euros_formatted = euros
        .abs()
        .to_string()
        .chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(".")
        .chars()
        .rev()
        .collect::<String>();

    let sign = if cents < 0 { "-" } else { "" };
    format!("{sign}{euros_formatted},{cents_part:02} EUR")
}

pub fn parse_euro_to_cents(input: &str) -> AppResult<i64> {
    let cleaned = input.trim().replace('.', "").replace(',', ".");
    let value: f64 = cleaned.parse().map_err(|_| {
        AppError::BadRequest("Bitte geben Sie einen gültigen Geldbetrag ein.".to_string())
    })?;

    Ok((value * 100.0).round() as i64)
}

pub fn normalize_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn normalize_email(input: &str) -> String {
    normalize_whitespace(input).to_lowercase()
}

pub fn normalize_city(input: &str) -> String {
    normalize_whitespace(input)
}

pub fn normalize_phone(input: &str) -> String {
    normalize_whitespace(input)
}

pub fn password_policy_errors(password: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if password.len() < 12 {
        errors.push("Das Passwort muss mindestens 12 Zeichen enthalten.".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        errors.push("Das Passwort muss mindestens einen Großbuchstaben enthalten.".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        errors.push("Das Passwort muss mindestens einen Kleinbuchstaben enthalten.".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("Das Passwort muss mindestens eine Zahl enthalten.".to_string());
    }
    if !password.chars().any(|c| !c.is_ascii_alphanumeric()) {
        errors.push("Das Passwort muss mindestens ein Sonderzeichen enthalten.".to_string());
    }
    errors
}

pub fn appointment_status_label(status: &str) -> &'static str {
    match status {
        "wartet_auf_email" => "Wartet auf E-Mail-Bestätigung",
        "angefragt" => "Angefragt",
        "bestaetigt" => "Bestätigt",
        "abgeschlossen" => "Abgeschlossen",
        "storniert" => "Storniert",
        _ => "Unbekannt",
    }
}

pub fn payment_status_label(status: &str) -> &'static str {
    match status {
        "offen" => "Offen",
        "teilweise_bezahlt" => "Teilweise bezahlt",
        "bezahlt" => "Bezahlt",
        _ => "Unbekannt",
    }
}

pub fn infer_payment_status(total_cents: i64, paid_cents: i64) -> &'static str {
    if paid_cents <= 0 {
        "offen"
    } else if paid_cents >= total_cents {
        "bezahlt"
    } else {
        "teilweise_bezahlt"
    }
}
