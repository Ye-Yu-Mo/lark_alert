use serde::Serialize;

/// A Feishu custom bot text message.
///
/// Serializes to:
/// ```json
/// {"msg_type":"text","content":{"text":"..."}}
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextMessage {
    msg_type: &'static str,
    content: TextContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TextContent {
    text: String,
}

impl TextMessage {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            msg_type: "text",
            content: TextContent { text: text.into() },
        }
    }

    pub fn builder() -> TextMessageBuilder {
        TextMessageBuilder::default()
    }

    pub fn text(&self) -> &str {
        &self.content.text
    }
}

#[derive(Debug, Default)]
pub struct TextMessageBuilder {
    text: String,
}

impl TextMessageBuilder {
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn build(self) -> TextMessage {
        TextMessage::new(self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_message_serializes_to_feishu_json() {
        let msg = TextMessage::new("hello");
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"msg_type":"text","content":{"text":"hello"}}"#);
    }

    #[test]
    fn text_message_builder() {
        let msg = TextMessage::builder().text("world").build();
        assert_eq!(msg.text(), "world");
    }

    #[test]
    fn text_message_allows_empty_text_for_api_compat() {
        // Empty is allowed at model level; the client may still send it.
        let msg = TextMessage::new("");
        assert_eq!(msg.text(), "");
    }
}
