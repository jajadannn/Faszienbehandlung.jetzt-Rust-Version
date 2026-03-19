use crate::views::{FaqItemView, InfoCard, ServiceCardView, StatCard};

pub fn home_stats() -> Vec<StatCard> {
    vec![
        StatCard {
            value: "45 Min.".to_string(),
            label: "sorgfaeltige Erstaufnahme mit Zeit fuer Ihre Fragen".to_string(),
        },
        StatCard {
            value: "ab 89 EUR".to_string(),
            label: "transparenter Einstiegspreis fuer den Ersttermin".to_string(),
        },
        StatCard {
            value: "1 Praxis".to_string(),
            label: "ruhiger, persoenlicher Rahmen ohne Massenabfertigung".to_string(),
        },
    ]
}

pub fn complaint_cards() -> Vec<InfoCard> {
    vec![
        InfoCard {
            title: "Nacken und Schultern".to_string(),
            text: "Wenn Spannungsgefuehle, Bildschirmarbeit oder einseitige Belastung Ihren Alltag dominieren, betrachten wir die umliegenden Faszienketten im Zusammenhang.".to_string(),
            accent: "sky".to_string(),
        },
        InfoCard {
            title: "Ruecken und Lendenbereich".to_string(),
            text: "Die Behandlung richtet sich auf Beweglichkeit, Koerperwahrnehmung und entlastende Reize, ohne unhaltbare Wirkversprechen zu machen.".to_string(),
            accent: "plum".to_string(),
        },
        InfoCard {
            title: "Beine, Huften, Sport".to_string(),
            text: "Nach intensiver Aktivitaet oder laengerem Sitzen kann eine strukturierte manuelle Begleitung helfen, Spannungsmuster besser einzuordnen.".to_string(),
            accent: "sky".to_string(),
        },
    ]
}

pub fn benefit_cards() -> Vec<InfoCard> {
    vec![
        InfoCard {
            title: "Sorgfaeltige Anamnese".to_string(),
            text: "Jeder Termin startet mit einer strukturierten Aufnahme Ihrer Situation, Ihrer Belastungsmuster und Ihrer Ziele im Alltag.".to_string(),
            accent: "plum".to_string(),
        },
        InfoCard {
            title: "Ruhiges Praxiserlebnis".to_string(),
            text: "Viel Weissraum, klare Kommunikation und ausreichend Zeit schaffen ein Umfeld, das Vertrauen und Orientierung foerdert.".to_string(),
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
            text: "Formulare, Kundenkonto, Session-Verwaltung und Rollenrechte sind fuer eine spaetere Live-Nutzung sauber vorbereitet.".to_string(),
            accent: "sky".to_string(),
        },
        InfoCard {
            title: "Klare Preisstruktur".to_string(),
            text: "Leistungen und Preisrahmen werden transparent kommuniziert, damit Interessenten von Anfang an realistisch planen koennen.".to_string(),
            accent: "plum".to_string(),
        },
        InfoCard {
            title: "Persoenliche Betreuung".to_string(),
            text: "Die Sprache bleibt sachlich, lokal und zugewandt, damit die Webseite wie eine echte Praxis wirkt und nicht wie ein Baukasten.".to_string(),
            accent: "sky".to_string(),
        },
    ]
}

pub fn service_cards() -> Vec<ServiceCardView> {
    vec![
        ServiceCardView {
            title: "Ersttermin Faszienanalyse".to_string(),
            subtitle: "Ausfuehrliche Anamnese, manuelle Befundung, erster Behandlungsimpuls"
                .to_string(),
            price: "89 EUR".to_string(),
            bullets: vec![
                "ca. 45 Minuten".to_string(),
                "inklusive Kurzprotokoll fuer Ihr Kundenkonto".to_string(),
                "geeignet fuer den strukturierten Einstieg".to_string(),
            ],
        },
        ServiceCardView {
            title: "Folgetermin Behandlung".to_string(),
            subtitle:
                "Weiterfuehrende Behandlungseinheit mit Fokus auf Verlauf und Belastungsmuster"
                    .to_string(),
            price: "79 EUR".to_string(),
            bullets: vec![
                "ca. 35 Minuten".to_string(),
                "mit Verlaufseinordnung".to_string(),
                "auch im Paket planbar".to_string(),
            ],
        },
        ServiceCardView {
            title: "Intensivtermin Bewegung & Gewebe".to_string(),
            subtitle: "Laengere Einheit mit manuellen Impulsen und alltagsnahen Empfehlungen"
                .to_string(),
            price: "119 EUR".to_string(),
            bullets: vec![
                "ca. 60 Minuten".to_string(),
                "fuer komplexere Spannungsmuster".to_string(),
                "inklusive Dokumentation".to_string(),
            ],
        },
    ]
}

pub fn faq_items() -> Vec<FaqItemView> {
    vec![
        FaqItemView {
            question: "Ist Faszienbehandlung eine medizinische Heilbehandlung?".to_string(),
            answer: "Die Webseite beschreibt eine fachlich begleitete Behandlung in einer Praxisumgebung. Es werden bewusst keine Heilversprechen abgegeben. Bei akuten oder unklaren Beschwerden sollte zusaetzlich aerztlicher Rat eingeholt werden.".to_string(),
        },
        FaqItemView {
            question: "Wie laeuft der erste Termin ab?".to_string(),
            answer: "Zu Beginn besprechen wir Ihre aktuelle Situation, Belastungen und Ziele. Erst danach folgt die eigentliche Behandlung und eine kurze Einordnung der naechsten sinnvollen Schritte.".to_string(),
        },
        FaqItemView {
            question: "Muss ich mich registrieren, um online zu buchen?".to_string(),
            answer: "Ja. Fuer die sichere Zuordnung Ihrer Termine und die spaetere Einsicht im Kundenbereich wird ein geschuetztes Konto angelegt. Die E-Mail-Adresse muss bestaetigt werden.".to_string(),
        },
        FaqItemView {
            question: "Wie werden meine Daten verwendet?".to_string(),
            answer: "Es werden nur die Daten verarbeitet, die fuer Terminverwaltung, Kundenkonto und organisatorische Kommunikation erforderlich sind. Die Datenschutzhinweise sind als Grundgeruest bereits vorbereitet.".to_string(),
        },
        FaqItemView {
            question: "Kann ich Termine stornieren oder verschieben?".to_string(),
            answer: "Terminanfragen werden im Kundenkonto und im Admin-Bereich verwaltet. Fuer konkrete Storno- oder Umbuchungsregeln sollte vor Livegang eine verbindliche Praxisregel ergaenzt werden.".to_string(),
        },
        FaqItemView {
            question: "Wer sieht Zahlungsinformationen?".to_string(),
            answer: "Zahlungsdaten sind rollenbasiert getrennt. Kunden sehen nur die eigenen Daten, der Admin-Bereich nur mit Administratorrechten.".to_string(),
        },
    ]
}
