use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use url::Url;

use crate::error::LarkAlertError;
use crate::models::{Card, PostMessage, TextMessage};
use crate::sign;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_BASE_BACKOFF_MS: u64 = 100;
const DEFAULT_MAX_BACKOFF_MS: u64 = 2_000;

/// A Feishu custom bot webhook client.
///
/// The client owns a `ureq::Agent`, so connections are pooled and reused across
/// sends. It supports optional signing, timeouts and bounded retries with
/// exponential backoff.
#[derive(Clone)]
pub struct LarkAlert {
    webhook_url: String,
    secret: Option<String>,
    agent: ureq::Agent,
    timeout: Duration,
    max_retries: u32,
    base_backoff_ms: u64,
    max_backoff_ms: u64,
}

impl LarkAlert {
    pub fn new(webhook_url: impl Into<String>) -> Result<Self, LarkAlertError> {
        let url = webhook_url.into();
        Self::validate_webhook_url(&url)?;
        Ok(Self {
            webhook_url: url,
            secret: None,
            agent: build_agent(DEFAULT_TIMEOUT),
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
            base_backoff_ms: DEFAULT_BASE_BACKOFF_MS,
            max_backoff_ms: DEFAULT_MAX_BACKOFF_MS,
        })
    }

    pub fn builder(webhook_url: impl Into<String>) -> Result<LarkAlertBuilder, LarkAlertError> {
        Ok(LarkAlertBuilder::new(Self::new(webhook_url)))
    }

    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = Some(secret.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.agent = build_agent(timeout);
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_backoff(mut self, base_ms: u64, max_ms: u64) -> Self {
        self.base_backoff_ms = base_ms;
        self.max_backoff_ms = max_ms.max(base_ms);
        self
    }

    pub fn webhook_url(&self) -> &str {
        &self.webhook_url
    }

    pub fn send<T: Serialize>(&self, message: &T) -> Result<(), LarkAlertError> {
        let body = self.build_body(message)?;
        let mut last_error: Option<LarkAlertError> = None;

        for attempt in 0..=self.max_retries {
            match self.send_once(&body) {
                Ok(()) => return Ok(()),
                Err(err) => {
                    if !err.is_retryable() || attempt == self.max_retries {
                        return Err(if attempt == self.max_retries && err.is_retryable() {
                            LarkAlertError::RetryExhausted {
                                retries: self.max_retries,
                                source: Box::new(err),
                            }
                        } else {
                            err
                        });
                    }
                    last_error = Some(err);
                    let backoff = self.backoff(attempt);
                    std::thread::sleep(backoff);
                }
            }
        }

        Err(LarkAlertError::RetryExhausted {
            retries: self.max_retries,
            source: Box::new(
                last_error.unwrap_or(LarkAlertError::Http("request failed".to_string())),
            ),
        })
    }

    pub fn send_text(&self, text: impl Into<String>) -> Result<(), LarkAlertError> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(LarkAlertError::Validation(
                "text message must not be empty".to_string(),
            ));
        }

        // A conservative safety limit for custom-bot text messages. Long text is
        // truncated rather than dropped so an alert still reaches the channel.
        const MAX_TEXT_CHARS: usize = 4000;
        let text = if text.chars().count() > MAX_TEXT_CHARS {
            let truncated: String = text.chars().take(MAX_TEXT_CHARS).collect();
            format!("{truncated}\n...[truncated by lark_alert]")
        } else {
            text
        };

        self.send(&TextMessage::new(text))
    }

    pub fn send_post(&self, message: &PostMessage) -> Result<(), LarkAlertError> {
        self.send(message)
    }

    pub fn send_card(&self, card: &Card) -> Result<(), LarkAlertError> {
        card.validate()?;
        self.send(&card.to_message())
    }

    fn validate_webhook_url(url: &str) -> Result<(), LarkAlertError> {
        let parsed = Url::parse(url).map_err(|e| LarkAlertError::InvalidUrl(e.to_string()))?;
        match parsed.scheme() {
            "http" | "https" => Ok(()),
            scheme => Err(LarkAlertError::InvalidUrl(format!(
                "unsupported scheme `{scheme}`, expected http or https"
            ))),
        }
    }

    fn build_body<T: Serialize>(&self, message: &T) -> Result<Value, LarkAlertError> {
        let mut body = serde_json::to_value(message)?;
        if let Some(secret) = &self.secret {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| LarkAlertError::Validation(format!("system clock error: {e}")))?
                .as_millis()
                .to_string();
            let sign = sign::sign(&timestamp, secret)?;
            body["timestamp"] = Value::String(timestamp);
            body["sign"] = Value::String(sign);
        }
        Ok(body)
    }

    fn send_once(&self, body: &Value) -> Result<(), LarkAlertError> {
        let response = self
            .agent
            .post(&self.webhook_url)
            .header("Content-Type", "application/json")
            .send_json(body)
            .map_err(|err| match err {
                ureq::Error::StatusCode(status) => LarkAlertError::HttpStatus {
                    status,
                    body: String::new(),
                },
                other => LarkAlertError::Http(other.to_string()),
            })?;

        let status = response.status().as_u16();
        let text = response
            .into_body()
            .read_to_string()
            .map_err(|e| LarkAlertError::Http(format!("failed to read response body: {e}")))?;

        if !(200..300).contains(&status) {
            return Err(LarkAlertError::HttpStatus { status, body: text });
        }

        let parsed: FeishuResponse = serde_json::from_str(&text)
            .map_err(|e| LarkAlertError::InvalidResponse(format!("{e}: {text}")))?;

        if parsed.code != 0 {
            return Err(LarkAlertError::Business {
                code: parsed.code,
                msg: parsed.msg.unwrap_or_default(),
            });
        }

        Ok(())
    }

    fn backoff(&self, attempt: u32) -> Duration {
        let exp = self.base_backoff_ms.saturating_mul(1u64 << attempt.min(10));
        Duration::from_millis(exp.min(self.max_backoff_ms))
    }
}

/// Builder for [`LarkAlert`].
pub struct LarkAlertBuilder {
    inner: Result<LarkAlert, LarkAlertError>,
}

impl LarkAlertBuilder {
    fn new(alert: Result<LarkAlert, LarkAlertError>) -> Self {
        Self { inner: alert }
    }

    pub fn secret(mut self, secret: impl Into<String>) -> Self {
        if let Ok(alert) = &mut self.inner {
            *alert = alert.clone().with_secret(secret);
        }
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        if let Ok(alert) = &mut self.inner {
            *alert = alert.clone().with_timeout(timeout);
        }
        self
    }

    pub fn max_retries(mut self, max_retries: u32) -> Self {
        if let Ok(alert) = &mut self.inner {
            *alert = alert.clone().with_max_retries(max_retries);
        }
        self
    }

    pub fn backoff(mut self, base_ms: u64, max_ms: u64) -> Self {
        if let Ok(alert) = &mut self.inner {
            *alert = alert.clone().with_backoff(base_ms, max_ms);
        }
        self
    }

    pub fn build(self) -> Result<LarkAlert, LarkAlertError> {
        self.inner
    }
}

fn build_agent(timeout: Duration) -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .build();
    ureq::Agent::new_with_config(config)
}

#[derive(Debug, serde::Deserialize)]
struct FeishuResponse {
    code: i64,
    #[serde(default)]
    msg: Option<String>,
}

impl LarkAlertError {
    fn is_retryable(&self) -> bool {
        match self {
            LarkAlertError::Http(_) => true,
            LarkAlertError::HttpStatus { status, .. } => *status >= 500,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Card, CardElement, PostElement, PostMessage, Severity};
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    fn spawn_mock<F>(handler: F) -> String
    where
        F: Fn(Value, usize) -> (u16, String) + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let mut counter = 0usize;
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { break };
                if let Some(request) = read_request(&mut stream) {
                    let body = request
                        .split_once("\r\n\r\n")
                        .map(|(_, body)| body)
                        .unwrap_or("");
                    let value: Value = serde_json::from_str(body).unwrap_or(Value::Null);
                    let (status, response_body) = handler(value, counter);
                    counter += 1;
                    let _ = write_response(&mut stream, status, &response_body);
                }
            }
        });
        format!("http://{addr}")
    }

    fn read_request(stream: &mut TcpStream) -> Option<String> {
        let mut data = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = stream.read(&mut buf).ok()?;
            if n == 0 {
                return None;
            }
            data.extend_from_slice(&buf[..n]);
            if let Some(end) = find_header_end(&data) {
                let header = String::from_utf8_lossy(&data[..end]);
                let content_length = header
                    .lines()
                    .find_map(|line| {
                        let (k, v) = line.split_once(':')?;
                        k.eq_ignore_ascii_case("content-length")
                            .then(|| v.trim().parse::<usize>().ok())
                            .flatten()
                    })
                    .unwrap_or(0);
                let total = end + 4 + content_length;
                while data.len() < total {
                    let n = stream.read(&mut buf).ok()?;
                    if n == 0 {
                        return None;
                    }
                    data.extend_from_slice(&buf[..n]);
                }
                return Some(String::from_utf8_lossy(&data).into_owned());
            }
        }
    }

    fn find_header_end(data: &[u8]) -> Option<usize> {
        data.windows(4).position(|w| w == b"\r\n\r\n")
    }

    fn write_response(stream: &mut TcpStream, status: u16, body: &str) -> std::io::Result<()> {
        let reason = if status == 200 { "OK" } else { "Error" };
        write!(
            stream,
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn ok_response() -> (u16, String) {
        (200, r#"{"code":0,"msg":"success"}"#.to_string())
    }

    #[test]
    fn rejects_invalid_webhook_url() {
        assert!(matches!(
            LarkAlert::new("not-a-url"),
            Err(LarkAlertError::InvalidUrl(_))
        ));
        assert!(matches!(
            LarkAlert::new("ftp://example.com/hook"),
            Err(LarkAlertError::InvalidUrl(_))
        ));
    }

    #[test]
    fn sends_text_without_secret() {
        let url = spawn_mock(|body, _| {
            assert_eq!(
                body,
                serde_json::json!({
                    "msg_type": "text",
                    "content": {"text": "hello"}
                })
            );
            ok_response()
        });
        let alert = LarkAlert::new(url).unwrap();
        alert.send_text("hello").unwrap();
    }

    #[test]
    fn sends_post_message() {
        let url = spawn_mock(|body, _| {
            assert_eq!(body["msg_type"], "post");
            assert_eq!(body["content"]["post"]["zh_cn"]["title"], "deploy");
            ok_response()
        });
        let alert = LarkAlert::new(url).unwrap();
        let post = PostMessage::builder()
            .title("deploy")
            .line(vec![PostElement::Text {
                text: "done".to_string(),
            }])
            .build();
        alert.send_post(&post).unwrap();
    }

    #[test]
    fn sends_card_message() {
        let url = spawn_mock(|body, _| {
            assert_eq!(body["msg_type"], "interactive");
            assert_eq!(body["card"]["header"]["template"], "red");
            ok_response()
        });
        let alert = LarkAlert::new(url).unwrap();
        let card = Card::new("svc", "node-1", "2026-01-01T00:00:00Z", "boom")
            .severity(Severity::Error)
            .title("test")
            .element(CardElement::Hr)
            .build();
        alert.send_card(&card).unwrap();
    }

    #[test]
    fn adds_signature_when_secret_configured() {
        let captured = Arc::new(std::sync::Mutex::new(None));
        let captured_clone = Arc::clone(&captured);
        let url = spawn_mock(move |body, _| {
            *captured_clone.lock().unwrap() = Some(body);
            ok_response()
        });
        let alert = LarkAlert::new(url).unwrap().with_secret("test_secret");
        alert.send_text("hello").unwrap();

        let body = captured.lock().unwrap().take().unwrap();
        let timestamp = body["timestamp"].as_str().unwrap();
        let sign = body["sign"].as_str().unwrap();
        assert_eq!(sign::sign(timestamp, "test_secret").unwrap(), sign);
        assert!(body.get("timestamp").is_some());
        assert!(body.get("sign").is_some());
    }

    #[test]
    fn rejects_empty_text() {
        let alert = LarkAlert::new("http://127.0.0.1:1").unwrap();
        let err = alert.send_text("   ").unwrap_err();
        assert!(matches!(err, LarkAlertError::Validation(_)));
    }

    #[test]
    fn truncates_long_text() {
        let captured = Arc::new(std::sync::Mutex::new(None));
        let captured_clone = Arc::clone(&captured);
        let url = spawn_mock(move |body, _| {
            *captured_clone.lock().unwrap() = Some(body);
            ok_response()
        });
        let alert = LarkAlert::new(url).unwrap();
        let long = "x".repeat(5000);
        alert.send_text(long).unwrap();
        let body = captured.lock().unwrap().take().unwrap();
        let text = body["content"]["text"].as_str().unwrap();
        assert!(text.len() < 4100);
        assert!(text.ends_with("[truncated by lark_alert]"));
    }

    #[test]
    fn maps_business_error() {
        let url = spawn_mock(|_, _| (200, r#"{"code":19001,"msg":"sign not match"}"#.to_string()));
        let alert = LarkAlert::new(url).unwrap();
        let err = alert.send_text("hello").unwrap_err();
        assert!(matches!(
            err,
            LarkAlertError::Business { code: 19001, msg } if msg == "sign not match"
        ));
    }

    #[test]
    fn retries_transient_failure_then_succeeds() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = Arc::clone(&attempts);
        let url = spawn_mock(move |_, counter| {
            attempts_clone.store(counter + 1, Ordering::SeqCst);
            if counter == 0 {
                (500, r#"{"code":1,"msg":"boom"}"#.to_string())
            } else {
                ok_response()
            }
        });
        let alert = LarkAlert::new(url)
            .unwrap()
            .with_max_retries(2)
            .with_backoff(0, 0);
        alert.send_text("hello").unwrap();
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn retry_exhausted_returns_last_error() {
        let url = spawn_mock(|_, _| (500, r#"{"code":1,"msg":"boom"}"#.to_string()));
        let alert = LarkAlert::new(url)
            .unwrap()
            .with_max_retries(1)
            .with_backoff(0, 0);
        let err = alert.send_text("hello").unwrap_err();
        assert!(matches!(
            err,
            LarkAlertError::RetryExhausted { retries: 1, .. }
        ));
    }
}
