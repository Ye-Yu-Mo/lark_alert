import json

import pytest

from lark_alert import Card, LarkAlert, PostMessage, Severity, TextMessage


def test_text_message_json():
    msg = TextMessage("hello")
    assert json.loads(msg.to_json()) == {
        "msg_type": "text",
        "content": {"text": "hello"},
    }


def test_post_message_json():
    msg = PostMessage("title").text_line("line")
    data = json.loads(msg.to_json())
    assert data["msg_type"] == "post"
    assert data["content"]["post"]["zh_cn"]["title"] == "title"
    assert data["content"]["post"]["zh_cn"]["content"] == [[{"tag": "text", "text": "line"}]]


def test_card_builder_and_color_mapping():
    card = (
        Card("api", "node-1", "2026-01-01T00:00:00Z", "no space left")
        .severity(Severity.Critical)
        .title("disk full")
        .summary("no space")
        .environment("prod")
        .field("host", "10.0.0.1")
        .wide_field("mount", "/")
    )
    data = json.loads(card.to_json())
    assert data["msg_type"] == "interactive"
    assert data["card"]["header"]["template"] == "carmine"
    assert data["card"]["header"]["title"]["content"] == "disk full"
    fields = [
        field
        for element in data["card"]["body"]["elements"]
        if "fields" in element
        for field in element["fields"]
    ]
    assert fields
    assert fields[-1]["is_short"] is False
    assert any(field["text"]["content"].startswith("**服务**") for field in fields)
    assert any(field["text"]["content"].startswith("**节点**") for field in fields)


def test_card_rejects_empty_required_fields():
    with pytest.raises(ValueError):
        Card("", "node-1", "2026-01-01T00:00:00Z", "content").to_json()


def test_invalid_webhook_raises_value_error_not_panic():
    with pytest.raises(ValueError):
        LarkAlert("not-a-url")


def test_client_uses_local_mock_server():
    import threading
    from http.server import BaseHTTPRequestHandler, HTTPServer

    received = {}

    class Handler(BaseHTTPRequestHandler):
        def do_POST(self):
            length = int(self.headers.get("Content-Length", "0"))
            body = self.rfile.read(length).decode()
            received["body"] = json.loads(body)
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(b'{"code":0,"msg":"success"}')

        def log_message(self, *args):
            pass

    server = HTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    url = f"http://127.0.0.1:{server.server_port}"

    alert = LarkAlert(url, secret="test_secret", timeout_secs=5, max_retries=1)
    alert.send_text("python mock")
    server.shutdown()

    assert received["body"]["msg_type"] == "text"
    assert "timestamp" in received["body"]
    assert "sign" in received["body"]


def test_send_text_rejects_empty_with_value_error():
    with pytest.raises(ValueError):
        LarkAlert("http://127.0.0.1:1").send_text("")
