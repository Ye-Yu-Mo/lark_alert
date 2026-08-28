use serde::Serialize;

/// A Feishu custom bot rich text (post) message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostMessage {
    msg_type: &'static str,
    content: PostContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PostContent {
    post: Post,
}

/// The post body. Feishu uses language-keyed maps; this library currently
/// generates the `zh_cn` locale for simplicity and future compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Post {
    #[serde(rename = "zh_cn")]
    zh_cn: PostLocale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PostLocale {
    title: String,
    content: Vec<Vec<PostElement>>,
}

/// One inline element inside a rich-text line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "tag", rename_all = "snake_case")]
pub enum PostElement {
    Text { text: String },
    A { text: String, href: String },
    At { user_id: String },
}

impl PostMessage {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            msg_type: "post",
            content: PostContent {
                post: Post {
                    zh_cn: PostLocale {
                        title: title.into(),
                        content: Vec::new(),
                    },
                },
            },
        }
    }

    pub fn builder() -> PostMessageBuilder {
        PostMessageBuilder::new("")
    }

    pub fn title(&self) -> &str {
        &self.content.post.zh_cn.title
    }

    pub fn lines(&self) -> &[Vec<PostElement>] {
        &self.content.post.zh_cn.content
    }

    /// Append a simple single-element text line.
    pub fn text_line(mut self, text: impl Into<String>) -> Self {
        self.content
            .post
            .zh_cn
            .content
            .push(vec![PostElement::Text { text: text.into() }]);
        self
    }

    /// Append a rich line made of multiple inline elements.
    pub fn line(mut self, elements: Vec<PostElement>) -> Self {
        self.content.post.zh_cn.content.push(elements);
        self
    }
}

#[derive(Debug, Clone)]
pub struct PostMessageBuilder {
    title: String,
    lines: Vec<Vec<PostElement>>,
}

impl PostMessageBuilder {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn line(mut self, elements: Vec<PostElement>) -> Self {
        self.lines.push(elements);
        self
    }

    pub fn text_line(mut self, text: impl Into<String>) -> Self {
        self.lines
            .push(vec![PostElement::Text { text: text.into() }]);
        self
    }

    pub fn build(self) -> PostMessage {
        PostMessage {
            msg_type: "post",
            content: PostContent {
                post: Post {
                    zh_cn: PostLocale {
                        title: self.title,
                        content: self.lines,
                    },
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_message_serializes_to_feishu_json() {
        let msg = PostMessage::builder()
            .title("deploy finished")
            .text_line("app=v1.2.3")
            .line(vec![
                PostElement::Text {
                    text: "docs: ".to_string(),
                },
                PostElement::A {
                    text: "link".to_string(),
                    href: "https://example.com".to_string(),
                },
            ])
            .build();

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["msg_type"], "post");
        assert_eq!(json["content"]["post"]["zh_cn"]["title"], "deploy finished");
        assert_eq!(
            json["content"]["post"]["zh_cn"]["content"][0][0]["tag"],
            "text"
        );
        assert_eq!(
            json["content"]["post"]["zh_cn"]["content"][1][1],
            serde_json::json!({"tag": "a", "text": "link", "href": "https://example.com"})
        );
    }

    #[test]
    fn post_message_builder_with_text_lines() {
        let msg = PostMessage::builder()
            .title("t")
            .text_line("a")
            .text_line("b")
            .build();
        assert_eq!(msg.lines().len(), 2);
    }
}
