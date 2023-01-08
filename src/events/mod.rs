use async_trait::async_trait;
use pyo3::{prelude::*, types::*};

use ricq::handler::{Handler, QEvent};

pub mod converter;
pub mod structs;
use crate::repr;
use structs::MessageSource;

use self::structs::MemberInfo;

#[pyclass]
#[derive(Debug, Clone)]
pub struct LoginEvent {
    #[pyo3(get)]
    uin: i64,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct GroupMessage {
    #[pyo3(get)]
    source: MessageSource,
    #[pyo3(get)]
    content: Py<PyAny>, // PyMessageChain
    #[pyo3(get)]
    sender: MemberInfo,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct FriendMessage {
    #[pyo3(get)]
    source: MessageSource,
    #[pyo3(get)]
    content: Py<PyAny>, // PyMessageChain
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct TempMessage {
    #[pyo3(get)]
    source: MessageSource,
    #[pyo3(get)]
    content: Py<PyAny>, // PyMessageChain
    #[pyo3(get)]
    sender: MemberInfo,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct UnknownEvent {
    inner: QEvent,
}

#[pymethods]
impl UnknownEvent {
    fn inner_repr(&self) -> String {
        format!("{:?}", self.inner)
    }
}

repr!(
    LoginEvent,
    GroupMessage,
    FriendMessage,
    TempMessage,
    UnknownEvent
);

pub struct PyHandler {
    callbacks: Py<PyList>,
}

impl PyHandler {
    pub fn new(callbacks: Py<PyList>) -> Self {
        Self { callbacks }
    }
}

#[async_trait]
impl Handler for PyHandler {
    async fn handle(&self, event: QEvent) {
        let event_repr = format!("{:?}", event);
        let py_event = match self::converter::convert(event).await {
            Ok(obj) => obj,
            Err(e) => {
                tracing::error!("转换事件 {} 时失败:", event_repr);
                Python::with_gil(|py| e.print_and_set_sys_last_vars(py));
                return;
            }
        };
        Python::with_gil(|py| {
            let args: Py<PyTuple> = PyTuple::new(py, &[py_event]).into_py(py);
            for cb in self.callbacks.as_ref(py) {
                match cb.call1(args.clone().as_ref(py)) {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("调用回调 {:?} 时失败:", cb);
                        e.print_and_set_sys_last_vars(py)
                    }
                }
            }
        })
    }
}
