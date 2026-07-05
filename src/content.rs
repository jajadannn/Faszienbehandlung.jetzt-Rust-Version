use crate::views::{FaqItemView, PracticeView, ServiceCardView};

pub fn service_cards(practice: &PracticeView) -> Vec<ServiceCardView> {
    vec![
        ServiceCardView {
            title: "Einzelbehandlung in der Praxis".to_string(),
            subtitle: format!(
                "Persönliche Sitzung in {} mit ruhigem Erstgespräch, manueller Behandlung und klarer Einordnung.",
                practice.region_label
            ),
            price: practice.single_session_price_label.clone(),
            bullets: vec![
                practice.appointment_duration_verbose.clone(),
                "inklusive Anamnese und individueller Hinweise".to_string(),
                format!("Praxisstandort: {}", practice.address_line_2),
            ],
        },
        ServiceCardView {
            title: practice.package_card_label.clone(),
            subtitle:
                "Für wiederkehrende Termine mit planbarer Preisstruktur und derselben ruhigen Praxisführung."
                    .to_string(),
            price: practice.package_session_price_label.clone(),
            bullets: vec![
                practice.package_validity_label.clone(),
                practice.package_savings_label.clone(),
                "ideal für fortlaufende Begleitung".to_string(),
            ],
        },
        ServiceCardView {
            title: "Hausbesuch in der Region".to_string(),
            subtitle: format!(
                "Nach Verfügbarkeit auch mobil in {} mit derselben strukturierten Terminlogik.",
                practice.house_call_area
            ),
            price: format!(
                "{} + {} Fahrtpauschale",
                practice.single_session_price_label, practice.house_call_fee_label
            ),
            bullets: vec![
                practice.appointment_duration_verbose.clone(),
                format!("zusätzliche Fahrtpauschale: {}", practice.house_call_fee_label),
                "nur nach individueller Abstimmung".to_string(),
            ],
        },
    ]
}

pub fn faq_items(practice: &PracticeView) -> Vec<FaqItemView> {
    vec![
        FaqItemView {
            question: "Ist Faszienbehandlung eine medizinische Heilbehandlung?".to_string(),
            answer: "Die Webseite beschreibt eine fachlich begleitete Behandlung in einer Praxisumgebung. Es werden bewusst keine Heilversprechen abgegeben. Bei akuten oder unklaren Beschwerden sollte zusätzlich ärztlicher Rat eingeholt werden.".to_string(),
        },
        FaqItemView {
            question: "Wie läuft der erste Termin ab?".to_string(),
            answer: format!(
                "Zu Beginn besprechen wir Ihre aktuelle Situation, Belastungen und Ziele. Anschließend folgt die eigentliche Behandlung. Eine Sitzung dauert in dieser Praxis {}.",
                practice.appointment_duration_verbose
            ),
        },
        FaqItemView {
            question: "Muss ich mich registrieren, um online zu buchen?".to_string(),
            answer: "Ja. Für die sichere Zuordnung Ihrer Termine und die spätere Einsicht im Kundenbereich wird ein geschütztes Konto angelegt. Die E-Mail-Adresse muss bestätigt werden.".to_string(),
        },
        FaqItemView {
            question: "Wie werden meine Daten verwendet?".to_string(),
            answer: "Es werden nur die Daten verarbeitet, die für Terminverwaltung, Kundenkonto und organisatorische Kommunikation erforderlich sind. Die Datenschutzhinweise sind als Grundgerüst bereits vorbereitet.".to_string(),
        },
        FaqItemView {
            question: "Kann ich Termine stornieren oder verschieben?".to_string(),
            answer: "Terminanfragen werden im Kundenkonto und im Admin-Bereich verwaltet. Für konkrete Storno- oder Umbuchungsregeln sollte vor Livegang eine verbindliche Praxisregel ergänzt werden.".to_string(),
        },
        FaqItemView {
            question: "Welche Preisangaben gelten auf der Webseite?".to_string(),
            answer: format!(
                "Die Einzelbehandlung ist aktuell mit {} ausgewiesen. Wiederkehrende Termine können über {} abgebildet werden. Maßgeblich bleiben die tatsächlich von der Praxis kommunizierten Konditionen.",
                practice.single_session_price_label, practice.package_card_label
            ),
        },
        FaqItemView {
            question: "Wo findet die Behandlung statt und wie erreiche ich die Praxis?".to_string(),
            answer: format!(
                "Die Praxis befindet sich in {}, {}. Öffnungszeiten: {}. Hausbesuche sind in {} nach Absprache möglich.",
                practice.address_line_1,
                practice.address_line_2,
                practice.opening_hours_summary,
                practice.house_call_area
            ),
        },
        FaqItemView {
            question: "Wer sieht Zahlungsinformationen?".to_string(),
            answer: "Zahlungsdaten sind rollenbasiert getrennt. Kunden sehen nur die eigenen Daten, der Admin-Bereich nur mit Administratorrechten.".to_string(),
        },
    ]
}
