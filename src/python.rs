//! PyO3 bindings for the `lark_alert` Rust core.
//!
//! These bindings are compiled only when the `python` feature is enabled
//! (which is what maturin does when building the extension module).

use std::time::Duration;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

use crate::client;
use crate::error::LarkAlertError;
use crate::models as rust;

/// Python-visible severity enum.
#[pyclass(
    name = "Severity",
    module = "lark_alert",
    eq,
    eq_int,
    frozen,
    from_py_object
)]
#[derive(Clone, Copy, PartialEq)]
pub enum PySeverity {
    Info,
    Success,
    Warning,
    Error,
    Critical,
}

impl From<PySeverity> for rust::Severity {
    fn from(value: PySeverity) -> Self {
        match value {
            PySeverity::Info => rust::Severity::Info,
            PySeverity::Success => rust::Severity::Success,
            PySeverity::Warning => rust::Severity::Warning,
            PySeverity::Error => rust::Severity::Error,
            PySeverity::Critical => rust::Severity::Critical,
        }
    }
}

#[pymethods]
impl PySeverity {
    /// Feishu card header template color for this severity.
    #[getter]
    fn color(&self) -> &'static str {
        rust::Severity::from(*self).color()
    }

    /// Stable lowercase name, e.g. `"error"`.
    #[getter]
    fn name(&self) -> &'static str {
        rust::Severity::from(*self).as_str()
    }
}

fn map_err(err: LarkAlertError) -> PyErr {
    match err {
        LarkAlertError::InvalidUrl(_) | LarkAlertError::Validation(_) => {
            PyValueError::new_err(err.to_string())
        }
        _ => PyRuntimeError::new_err(err.to_string()),
    }
}

/// A typed Feishu interactive card with the unified alert style.
#[pyclass(name = "Card", module = "lark_alert", from_py_object)]
#[derive(Clone)]
pub struct PyCard {
    inner: rust::Card,
}

#[pymethods]
impl PyCard {
    #[new]
    fn new() -> Self {
        Self {
            inner: rust::Card::new(),
        }
    }

    fn severity<'py>(&self, py: Python<'py>, severity: PySeverity) -> PyResult<Py<PyCard>> {
        let inner = self.inner.clone().severity(severity.into());
        Py::new(py, PyCard { inner })
    }

    fn title<'py>(&self, py: Python<'py>, title: &str) -> PyResult<Py<PyCard>> {
        let inner = self.inner.clone().title(title.to_string());
        Py::new(py, PyCard { inner })
    }

    fn summary<'py>(&self, py: Python<'py>, summary: &str) -> PyResult<Py<PyCard>> {
        let inner = self.inner.clone().summary(summary.to_string());
        Py::new(py, PyCard { inner })
    }

    fn service<'py>(&self, py: Python<'py>, service: &str) -> PyResult<Py<PyCard>> {
        let inner = self.inner.clone().service(service.to_string());
        Py::new(py, PyCard { inner })
    }

    fn environment<'py>(&self, py: Python<'py>, environment: &str) -> PyResult<Py<PyCard>> {
        let inner = self.inner.clone().environment(environment.to_string());
        Py::new(py, PyCard { inner })
    }

    fn timestamp<'py>(&self, py: Python<'py>, timestamp: &str) -> PyResult<Py<PyCard>> {
        let inner = self.inner.clone().timestamp(timestamp.to_string());
        Py::new(py, PyCard { inner })
    }

    /// Alias for `timestamp`.
    fn time<'py>(&self, py: Python<'py>, timestamp: &str) -> PyResult<Py<PyCard>> {
        let inner = self.inner.clone().time(timestamp.to_string());
        Py::new(py, PyCard { inner })
    }

    fn details<'py>(&self, py: Python<'py>, details: &str) -> PyResult<Py<PyCard>> {
        let inner = self.inner.clone().details(details.to_string());
        Py::new(py, PyCard { inner })
    }

    fn note<'py>(&self, py: Python<'py>, note: &str) -> PyResult<Py<PyCard>> {
        let inner = self.inner.clone().note(note.to_string());
        Py::new(py, PyCard { inner })
    }

    fn field<'py>(&self, py: Python<'py>, label: &str, value: &str) -> PyResult<Py<PyCard>> {
        let inner = self
            .inner
            .clone()
            .field(label.to_string(), value.to_string());
        Py::new(py, PyCard { inner })
    }

    fn wide_field<'py>(&self, py: Python<'py>, label: &str, value: &str) -> PyResult<Py<PyCard>> {
        let inner = self
            .inner
            .clone()
            .wide_field(label.to_string(), value.to_string());
        Py::new(py, PyCard { inner })
    }

    /// Serialize this card (including the `msg_type` wrapper) to a JSON string.
    fn to_json(&self) -> PyResult<String> {
        self.inner
            .to_json()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!(
            "Card(title={:?}, severity={:?})",
            self.inner.title_value(),
            self.inner.severity_value().as_str()
        )
    }
}

/// A text message suitable for Feishu custom bots.
#[pyclass(name = "TextMessage", module = "lark_alert", from_py_object)]
#[derive(Clone)]
pub struct PyTextMessage {
    inner: rust::TextMessage,
}

#[pymethods]
impl PyTextMessage {
    #[new]
    fn new(text: &str) -> Self {
        Self {
            inner: rust::TextMessage::new(text.to_string()),
        }
    }

    #[getter]
    fn text(&self) -> String {
        self.inner.text().to_string()
    }

    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

/// A rich text (post) message suitable for Feishu custom bots.
#[pyclass(name = "PostMessage", module = "lark_alert", from_py_object)]
#[derive(Clone)]
pub struct PyPostMessage {
    inner: rust::PostMessage,
}

#[pymethods]
impl PyPostMessage {
    #[new]
    fn new(title: &str) -> Self {
        Self {
            inner: rust::PostMessage::new(title.to_string()),
        }
    }

    fn text_line<'py>(&self, py: Python<'py>, text: &str) -> PyResult<Py<PyPostMessage>> {
        let inner = self.inner.clone().text_line(text.to_string());
        Py::new(py, PyPostMessage { inner })
    }

    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

/// Synchronous Feishu webhook client.
#[pyclass(name = "LarkAlert", module = "lark_alert", from_py_object)]
#[derive(Clone)]
pub struct PyLarkAlert {
    inner: client::LarkAlert,
}

#[pymethods]
impl PyLarkAlert {
    #[new]
    #[pyo3(signature = (webhook_url, secret=None, timeout_secs=None, max_retries=None))]
    fn new(
        webhook_url: &str,
        secret: Option<&str>,
        timeout_secs: Option<u64>,
        max_retries: Option<u32>,
    ) -> PyResult<Self> {
        let mut alert = client::LarkAlert::new(webhook_url.to_string()).map_err(map_err)?;
        if let Some(secret) = secret {
            alert = alert.with_secret(secret.to_string());
        }
        if let Some(timeout_secs) = timeout_secs {
            alert = alert.with_timeout(Duration::from_secs(timeout_secs));
        }
        if let Some(max_retries) = max_retries {
            alert = alert.with_max_retries(max_retries);
        }
        Ok(Self { inner: alert })
    }

    fn send_text<'py>(&self, py: Python<'py>, text: &str) -> PyResult<()> {
        let inner = self.inner.clone();
        let text = text.to_string();
        py.detach(move || inner.send_text(text)).map_err(map_err)
    }

    fn send_post<'py>(&self, py: Python<'py>, message: &PyPostMessage) -> PyResult<()> {
        let inner = self.inner.clone();
        let message = message.inner.clone();
        py.detach(move || inner.send_post(&message))
            .map_err(map_err)
    }

    fn send_card<'py>(&self, py: Python<'py>, card: &PyCard) -> PyResult<()> {
        let inner = self.inner.clone();
        let card = card.inner.clone();
        py.detach(move || inner.send_card(&card)).map_err(map_err)
    }
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLarkAlert>()?;
    m.add_class::<PyCard>()?;
    m.add_class::<PySeverity>()?;
    m.add_class::<PyTextMessage>()?;
    m.add_class::<PyPostMessage>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
