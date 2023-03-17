use async_trait::async_trait;
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_asyncio::{into_future_with_locals, TaskLocals};
use pyo3_repr::PyRepr;
use ricq::handler::{Handler, QEvent};

pub mod converter;
pub mod structs;
use structs::MessageSource;

use self::structs::{FriendInfo, MemberInfo};
use crate::utils::py_try;

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
pub struct GroupRecallMessage {
    time: PyObject, // PyDatetime
    author: MemberInfo,
    operator: MemberInfo,
    seq: i32,
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
pub struct FriendRecallMessage {
    time: PyObject, // PyDatetime
    author: FriendInfo,
    seq: i32,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct TempMessage {
    source: MessageSource,
    content: PyObject, // PyMessageChain
    sender: MemberInfo,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct GroupNudge {
    sender: MemberInfo,
    receiver: MemberInfo,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct FriendNudge {
    sender: FriendInfo,
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
    queues: Py<PyList>,
    locals: TaskLocals,
}

impl PyHandler {
    pub fn new(queues: Py<PyList>, locals: TaskLocals) -> Self {
        Self { queues, locals }
    }
}

#[async_trait]
impl Handler for PyHandler {
    async fn handle(&self, event: QEvent) {
        let event_repr = format!("{event:?}");
        let py_event = match self::converter::convert(event).await {
            Ok(obj) => obj,
            Err(e) => {
                tracing::error!("转换事件 {} 时失败: {}", event_repr, e);
                return;
            }
        };
        let mut handles: Vec<tokio::task::JoinHandle<Result<(), PyErr>>> = vec![];
        Python::with_gil(|py| {
            if py_event.is_none(py) {
                return;
            }
            let args: Py<PyTuple> = (py_event,).into_py(py);
            for q in self.queues.as_ref(py).iter().map(|q| q.into_py(py)) {
                let locals = self.locals.clone();
                let args = args.clone();
                handles.push(tokio::spawn(async move {
                    py_try(|py| {
                        into_future_with_locals(
                            &locals,
                            q.as_ref(py).getattr("put")?.call1(args.as_ref(py))?,
                        )
                    })?
                    .await?;
                    Ok(())
                }));
            }
        });
        for handle in handles {
            match handle.await {
                Err(err) => {
                    tracing::error!("向队列发送事件失败: {:?}", err);
                }
                Ok(Err(err)) => {
                    tracing::error!("向队列发送事件失败: {:?}", err);
                }
                Ok(Ok(())) => {}
            };
        }
    }
}
