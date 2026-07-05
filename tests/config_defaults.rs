use faszienbehandlung_jetzt::config::AppConfig;

#[test]
fn config_defaults_parse_without_env() {
    let config = AppConfig::from_env().expect("Defaults müssen ohne Umgebungsvariablen parsen");

    assert_eq!(config.appointment_duration_minutes, 90);
    assert_eq!(config.booking_package_validity_months, 12);
    assert!(config.booking_base_price_cents > 0);
    assert!(config.base_url.starts_with("http"));
    assert!(
        config.smtp_host.is_none(),
        "Ohne SMTP_HOST muss LogOnly greifen"
    );
}
