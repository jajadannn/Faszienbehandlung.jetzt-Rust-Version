use crate::views::{FaqItemView, InfoCard, PracticeView, ServiceCardView, StatCard};

pub fn home_stats() -> Vec<StatCard> {
    vec![
        StatCard {
            value: "45 Min.".to_string(),
            label: "sorgfältige Erstaufnahme mit Zeit für Ihre Fragen".to_string(),
        },
        StatCard {
            value: "ab 89 EUR".to_string(),
            label: "transparenter Einstiegspreis für den Ersttermin".to_string(),
        },
        StatCard {
            value: "1 Praxis".to_string(),
            label: "ruhiger, persönlicher Rahmen ohne Massenabfertigung".to_string(),
        },
    ]
}

pub fn complaint_cards() -> Vec<InfoCard> {
    vec![
        InfoCard {
            title: "Nacken und Schultern".to_string(),
            text: "Wenn Spannungsgefühle, Bildschirmarbeit oder einseitige Belastung Ihren Alltag dominieren, betrachten wir die umliegenden Faszienketten im Zusammenhang.".to_string(),
            accent: "sky".to_string(),
        },
        InfoCard {
            title: "Rücken und Lendenbereich".to_string(),
            text: "Die Behandlung richtet sich auf Beweglichkeit, Körperwahrnehmung und entlastende Reize, ohne unhaltbare Wirkversprechen zu machen.".to_string(),
            accent: "plum".to_string(),
        },
        InfoCard {
            title: "Beine, Hüften, Sport".to_string(),
            text: "Nach intensiver Aktivität oder längerem Sitzen kann eine strukturierte manuelle Begleitung helfen, Spannungsmuster besser einzuordnen.".to_string(),
            accent: "sky".to_string(),
        },
    ]
}

pub fn benefit_cards() -> Vec<InfoCard> {
    vec![
        InfoCard {
            title: "Sorgfältige Anamnese".to_string(),
            text: "Jeder Termin startet mit einer strukturierten Aufnahme Ihrer Situation, Ihrer Belastungsmuster und Ihrer Ziele im Alltag.".to_string(),
            accent: "plum".to_string(),
        },
        InfoCard {
            title: "Ruhiges Praxiserlebnis".to_string(),
            text: "Viel Weißraum, klare Kommunikation und ausreichend Zeit schaffen ein Umfeld, das Vertrauen und Orientierung fördert.".to_string(),
            accent: "sky".to_string(),
        },
        InfoCard {
            title: "Nachvollziehbare Empfehlungen".to_string(),
            text: "Sie erhalten nur Hinweise, die zur aktuellen Situation passen und ohne Heilversprechen auskommen.".to_string(),
            accent: "plum".to_string(),
        },
    ]
}

pub fn trust_cards() -> Vec<InfoCard> {
    vec![
        InfoCard {
            title: "Datenschutzbewusste Prozesse".to_string(),
            text: "Formulare, Kundenkonto, Session-Verwaltung und Rollenrechte sind für eine spätere Live-Nutzung sauber vorbereitet.".to_string(),
            accent: "sky".to_string(),
        },
        InfoCard {
            title: "Klare Preisstruktur".to_string(),
            text: "Leistungen und Preisrahmen werden transparent kommuniziert, damit Interessenten von Anfang an realistisch planen können.".to_string(),
            accent: "plum".to_string(),
        },
        InfoCard {
            title: "Persönliche Betreuung".to_string(),
            text: "Die Sprache bleibt sachlich, lokal und zugewandt, damit die Webseite wie eine echte Praxis wirkt und nicht wie ein Baukasten.".to_string(),
            accent: "sky".to_string(),
        },
    ]
}

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
