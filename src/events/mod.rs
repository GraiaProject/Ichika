use async_trait::async_trait;
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_repr::PyRepr;
use ricq::handler::{Handler, QEvent};

pub mod converter;
pub mod structs;
use structs::MessageSource;

use self::structs::{FriendInfo, MemberInfo};

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct LoginEvent {
    uin: i64,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct GroupMessage {
    source: MessageSource,
    content: PyObject, // PyMessageChain
    sender: MemberInfo,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct FriendMessage {
    source: MessageSource,
    content: PyObject, // PyMessageChain
    sender: FriendInfo,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct TempMessage {
    source: MessageSource,
    content: PyObject, // PyMessageChain
    sender: MemberInfo,
}

#[pyclass]
#[derive(PyRepr, Clone)]
pub struct UnknownEvent {
    inner: QEvent,
}

#[pymethods]
impl UnknownEvent {
    fn inner_repr(&self) -> String {
        format!("{:?}", self.inner)
    }
}

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
        let event_repr = format!("{event:?}");
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
                        e.print_and_set_sys_last_vars(py);
                    }
                }
            }
        });
    }
}
