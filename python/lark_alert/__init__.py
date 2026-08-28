"""Production-ready Feishu/Lark custom bot notifications.

The Rust implementation is exposed through the compiled ``lark_alert._core``
extension module; this package re-exports the public Python API.
"""

from ._core import __version__, Card, LarkAlert, PostMessage, Severity, TextMessage

__all__ = [
    "Card",
    "LarkAlert",
    "PostMessage",
    "Severity",
    "TextMessage",
    "__version__",
]
