use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::error::LarkAlertError;

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
///
/// Required alert context: service, node, timestamp and content.
/// These fields are part of the constructor so an alert cannot be created
/// without describing what happened, where it happened and when.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    severity: Severity,
    title: String,
    summary: Option<String>,
    service: String,
    node: String,
    environment: Option<String>,
    timestamp: String,
    content: String,
    fields: Vec<CardField>,
    details: Option<String>,
    note: Option<String>,
    custom_elements: Vec<CardElement>,
}

impl Card {
    pub fn new(
        service: impl Into<String>,
        node: impl Into<String>,
        timestamp: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Info,
            title: "Alert".to_string(),
            summary: None,
            service: service.into(),
            node: node.into(),
            environment: None,
            timestamp: timestamp.into(),
            content: content.into(),
            fields: Vec::new(),
            details: None,
            note: None,
            custom_elements: Vec::new(),
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
        self.service = service.into();
        self
    }

    pub fn node(mut self, node: impl Into<String>) -> Self {
        self.node = node.into();
        self
    }

    pub fn environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    pub fn timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = timestamp.into();
        self
    }

    /// Alias for [`Card::timestamp`].
    pub fn time(mut self, time: impl Into<String>) -> Self {
        self.timestamp = time.into();
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Alias for [`Card::content`].
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.content = message.into();
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

    /// Validate that all required alert context fields are present.
    ///
    /// The constructor already requires these values, but this method also
    /// rejects blank strings so a `Card` cannot be sent with empty context.
    pub fn validate(&self) -> Result<(), LarkAlertError> {
        for (field, value) in [
            ("service", self.service.as_str()),
            ("node", self.node.as_str()),
            ("timestamp", self.timestamp.as_str()),
            ("content", self.content.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(LarkAlertError::Validation(format!(
                    "card field `{field}` must not be empty"
                )));
            }
        }
        Ok(())
    }

    pub fn to_message(&self) -> CardMessage {
        CardMessage {
            msg_type: "interactive",
            card: self.clone(),
        }
    }

    pub fn to_json(&self) -> Result<String, LarkAlertError> {
        self.validate()?;
        Ok(serde_json::to_string(&self.to_message())?)
    }

    fn wire_elements(&self) -> Vec<WireElement> {
        let mut elements = Vec::new();

        if let Some(summary) = &self.summary {
            elements.push(WireElement::Div {
                text: Some(WireText::lark_md(summary)),
                fields: None,
            });
        }

        elements.push(WireElement::Div {
            text: Some(WireText::lark_md(&self.content)),
            fields: None,
        });

        let mut fields = Vec::new();
        fields.push(CardField::new("服务", &self.service));
        fields.push(CardField::new("节点", &self.node));
        if let Some(environment) = &self.environment {
            fields.push(CardField::new("环境", environment));
        }
        fields.push(CardField::new("时间", &self.timestamp));
        fields.extend(self.fields.clone());

        elements.push(WireElement::Div {
            text: None,
            fields: Some(fields),
        });

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
        let card = Card::new(
            "order-api",
            "node-1",
            "2026-01-01T00:00:00Z",
            "checkout is failing",
        )
        .severity(Severity::Error)
        .title("Order service down")
        .summary("checkout is failing")
        .environment("prod")
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
    fn card_includes_required_context_fields() {
        let card = Card::new("api", "node-1", "2026-01-01T00:00:00Z", "disk full")
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
        assert_eq!(fields.len(), 5);

        let contents: Vec<String> = fields
            .iter()
            .map(|f| f["text"]["content"].as_str().unwrap().to_string())
            .collect();
        assert!(contents.iter().any(|c| c.starts_with("**服务**\napi")));
        assert!(contents.iter().any(|c| c.starts_with("**节点**\nnode-1")));
        assert!(
            contents
                .iter()
                .any(|c| c.starts_with("**时间**\n2026-01-01T00:00:00Z"))
        );
    }

    #[test]
    fn custom_elements_are_appended() {
        let card = Card::new("api", "node-1", "2026-01-01T00:00:00Z", "content")
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
        let card = Card::new(
            "order-api",
            "node-1",
            "2026-01-01T00:00:00Z",
            "something failed",
        )
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
                        {"tag": "div", "text": {"tag": "lark_md", "content": "something failed"}},
                        {"tag": "div", "fields": [
                            {"is_short": true, "text": {"tag": "lark_md", "content": "**服务**\norder-api"}},
                            {"is_short": true, "text": {"tag": "lark_md", "content": "**节点**\nnode-1"}},
                            {"is_short": true, "text": {"tag": "lark_md", "content": "**时间**\n2026-01-01T00:00:00Z"}},
                            {"is_short": true, "text": {"tag": "lark_md", "content": "**a**\nb"}}
                        ]}
                    ]
                }
            }
        });
        assert_eq!(serde_json::to_value(card.to_message()).unwrap(), expected);
    }

    #[test]
    fn card_validate_rejects_empty_required_fields() {
        let card = Card::new("api", "node-1", "2026-01-01T00:00:00Z", "");
        assert!(matches!(
            card.validate(),
            Err(LarkAlertError::Validation(msg)) if msg.contains("content")
        ));

        let card = Card::new("", "node-1", "2026-01-01T00:00:00Z", "content");
        assert!(matches!(
            card.validate(),
            Err(LarkAlertError::Validation(msg)) if msg.contains("service")
        ));
    }
}
