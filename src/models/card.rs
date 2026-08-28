use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

/// Severity levels used by the unified card style.
///
/// The mapping is fixed and intentionally does **not** use emoji:
/// `info=blue`, `success=green`, `warning=orange`, `error=red`, `critical=carmine`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Severity {
    #[default]
    Info,
    Success,
    Warning,
    Error,
    Critical,
}

impl Severity {
    pub const fn color(self) -> &'static str {
        match self {
            Severity::Info => "blue",
            Severity::Success => "green",
            Severity::Warning => "orange",
            Severity::Error => "red",
            Severity::Critical => "carmine",
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Success => "success",
            Severity::Warning => "warning",
            Severity::Error => "error",
            Severity::Critical => "critical",
        }
    }
}

/// A field shown in the card's two-column layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardField {
    label: String,
    value: String,
    is_short: bool,
}

impl CardField {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            is_short: true,
        }
    }

    pub fn wide(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            is_short: false,
        }
    }
}

/// A custom card element. The library renders these after the default summary
/// and key-value fields, before the bottom note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardElement {
    Div { text: String },
    Fields(Vec<CardField>),
    Markdown { content: String },
    Hr,
    Note { content: String },
}

/// The unified interactive card message for Feishu custom bots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardMessage {
    msg_type: &'static str,
    card: Card,
}

/// Typed builder for Feishu interactive cards.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Card {
    severity: Severity,
    title: String,
    summary: Option<String>,
    service: Option<String>,
    environment: Option<String>,
    timestamp: Option<String>,
    fields: Vec<CardField>,
    details: Option<String>,
    note: Option<String>,
    custom_elements: Vec<CardElement>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            severity: Severity::Info,
            title: "Alert".to_string(),
            ..Self::default()
        }
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.service = Some(service.into());
        self
    }

    pub fn environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    pub fn timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Alias for [`Card::timestamp`].
    pub fn time(mut self, time: impl Into<String>) -> Self {
        self.timestamp = Some(time.into());
        self
    }

    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn field(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push(CardField::new(label, value));
        self
    }

    pub fn wide_field(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push(CardField::wide(label, value));
        self
    }

    pub fn element(mut self, element: CardElement) -> Self {
        self.custom_elements.push(element);
        self
    }

    /// Identity method; useful when a builder-style expression is required.
    pub fn build(self) -> Self {
        self
    }

    pub fn severity_value(&self) -> Severity {
        self.severity
    }

    pub fn title_value(&self) -> &str {
        &self.title
    }

    pub fn to_message(&self) -> CardMessage {
        CardMessage {
            msg_type: "interactive",
            card: self.clone(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.to_message())
    }

    fn wire_elements(&self) -> Vec<WireElement> {
        let mut elements = Vec::new();

        if let Some(summary) = &self.summary {
            elements.push(WireElement::Div {
                text: Some(WireText::lark_md(summary)),
                fields: None,
            });
        }

        let mut fields = self.fields.clone();
        if let Some(service) = &self.service {
            fields.insert(0, CardField::new("服务", service));
        }
        if let Some(environment) = &self.environment {
            fields.insert(0, CardField::new("环境", environment));
        }
        if let Some(timestamp) = &self.timestamp {
            fields.insert(0, CardField::new("时间", timestamp));
        }

        if !fields.is_empty() {
            elements.push(WireElement::Div {
                text: None,
                fields: Some(fields),
            });
        }

        for element in &self.custom_elements {
            elements.push(match element {
                CardElement::Div { text } => WireElement::Div {
                    text: Some(WireText::lark_md(text)),
                    fields: None,
                },
                CardElement::Fields(fields) => WireElement::Div {
                    text: None,
                    fields: Some(fields.clone()),
                },
                CardElement::Markdown { content } => WireElement::Markdown {
                    content: content.clone(),
                },
                CardElement::Hr => WireElement::Hr,
                CardElement::Note { content } => WireElement::Note {
                    elements: vec![WireText::plain_text(content)],
                },
            });
        }

        if self.details.is_some() {
            elements.push(WireElement::Hr);
        }
        if let Some(details) = &self.details {
            elements.push(WireElement::Div {
                text: Some(WireText::lark_md(details)),
                fields: None,
            });
        }

        if let Some(note) = &self.note {
            elements.push(WireElement::Note {
                elements: vec![WireText::plain_text(note)],
            });
        }

        elements
    }
}

impl Serialize for Card {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Card", 4)?;
        state.serialize_field("schema", "2.0")?;
        state.serialize_field(
            "config",
            &CardConfig {
                wide_screen_mode: true,
            },
        )?;
        state.serialize_field(
            "header",
            &CardHeader {
                template: self.severity.color(),
                title: WireText::plain_text(&self.title),
            },
        )?;
        state.serialize_field(
            "body",
            &CardBody {
                elements: self.wire_elements(),
            },
        )?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize)]
struct CardBody {
    elements: Vec<WireElement>,
}

#[derive(Debug, Clone, Serialize)]
struct CardConfig {
    wide_screen_mode: bool,
}

#[derive(Debug, Clone, Serialize)]
struct CardHeader {
    template: &'static str,
    title: WireText,
}

#[derive(Debug, Clone, Serialize)]
struct WireText {
    tag: &'static str,
    content: String,
}

impl WireText {
    fn plain_text(content: &str) -> Self {
        Self {
            tag: "plain_text",
            content: content.to_string(),
        }
    }

    fn lark_md(content: &str) -> Self {
        Self {
            tag: "lark_md",
            content: content.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "tag", rename_all = "snake_case")]
enum WireElement {
    Div {
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<WireText>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fields: Option<Vec<CardField>>,
    },
    Markdown {
        content: String,
    },
    Note {
        elements: Vec<WireText>,
    },
    Hr,
}

impl Serialize for CardField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CardField", 2)?;
        state.serialize_field("is_short", &self.is_short)?;
        state.serialize_field(
            "text",
            &WireText::lark_md(&format!("**{}**\n{}", self.label, self.value)),
        )?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn severity_color_mapping_is_fixed() {
        assert_eq!(Severity::Info.color(), "blue");
        assert_eq!(Severity::Success.color(), "green");
        assert_eq!(Severity::Warning.color(), "orange");
        assert_eq!(Severity::Error.color(), "red");
        assert_eq!(Severity::Critical.color(), "carmine");
    }

    #[test]
    fn card_uses_unified_template_without_emoji() {
        let card = Card::new()
            .severity(Severity::Error)
            .title("Order service down")
            .summary("checkout is failing")
            .service("order-api")
            .environment("prod")
            .timestamp("2026-01-01T00:00:00Z")
            .details("error rate > 5% for 10 minutes")
            .note("auto-generated by lark_alert")
            .build();

        let value = serde_json::to_value(card.to_message()).unwrap();

        assert_eq!(value["msg_type"], "interactive");
        assert_eq!(value["card"]["schema"], "2.0");
        assert_eq!(value["card"]["config"]["wide_screen_mode"], true);
        assert_eq!(value["card"]["header"]["template"], "red");
        assert_eq!(
            value["card"]["header"]["title"]["content"],
            "Order service down"
        );

        let elements = value["card"]["body"]["elements"].as_array().unwrap();
        let kinds: Vec<&str> = elements
            .iter()
            .map(|e| e["tag"].as_str().unwrap())
            .collect();
        assert!(kinds.contains(&"div"));
        assert!(kinds.contains(&"hr"));
        assert!(kinds.contains(&"note"));

        let serialized = card.to_json().unwrap();
        assert!(!serialized.contains("emoji"));
        assert!(!serialized.contains("🔴"));
    }

    #[test]
    fn default_card_has_two_column_fields() {
        let card = Card::new()
            .service("api")
            .environment("staging")
            .field("region", "cn")
            .build();
        let value = serde_json::to_value(card.to_message()).unwrap();
        let elements = value["card"]["body"]["elements"].as_array().unwrap();
        let fields_element = elements
            .iter()
            .find(|e| e["fields"].is_array())
            .expect("fields element should exist");
        let fields = fields_element["fields"].as_array().unwrap();
        assert!(fields.iter().all(|f| f["is_short"] == true));
        assert_eq!(fields.len(), 3);
    }

    #[test]
    fn custom_elements_are_appended() {
        let card = Card::new()
            .element(CardElement::Hr)
            .element(CardElement::Note {
                content: "custom".to_string(),
            })
            .build();
        let value = serde_json::to_value(card.to_message()).unwrap();
        let elements = value["card"]["body"]["elements"].as_array().unwrap();
        let last = elements.last().unwrap();
        assert_eq!(last["tag"], "note");
        assert_eq!(last["elements"][0]["content"], "custom");
    }

    #[test]
    fn card_json_matches_expected_shape() {
        let card = Card::new()
            .severity(Severity::Info)
            .title("title")
            .summary("summary")
            .field("a", "b")
            .build();
        let expected = json!({
            "msg_type": "interactive",
            "card": {
                "schema": "2.0",
                "config": {"wide_screen_mode": true},
                "header": {
                    "template": "blue",
                    "title": {"tag": "plain_text", "content": "title"}
                },
                "body": {
                    "elements": [
                        {"tag": "div", "text": {"tag": "lark_md", "content": "summary"}},
                        {"tag": "div", "fields": [
                            {"is_short": true, "text": {"tag": "lark_md", "content": "**a**\nb"}}
                        ]}
                    ]
                }
            }
        });
        assert_eq!(serde_json::to_value(card.to_message()).unwrap(), expected);
    }
}
