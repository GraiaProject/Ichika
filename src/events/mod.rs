use async_trait::async_trait;
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_asyncio::{into_future_with_locals, TaskLocals};
use pyo3_repr::PyRepr;
use ricq::client::event::DisconnectReason;
use ricq::client::NetworkStatus;
use ricq::handler::{Handler, QEvent};

pub mod converter;

use crate::utils::{datetime_from_ts, py_client_refs, py_try, py_use};

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct MessageSource {
    pub seq: i32,
    pub rand: i32,
    pub raw_seqs: Py<PyTuple>,
    pub raw_rands: Py<PyTuple>,
    pub time: PyObject,
}

impl MessageSource {
    pub fn new(py: Python, seqs: &[i32], rands: &[i32], time: i32) -> PyResult<Self> {
        let seq = *seqs
            .first()
            .ok_or_else(|| PyIndexError::new_err("Empty returning rands"))?;
        let rand = *rands
            .first()
            .ok_or_else(|| PyIndexError::new_err("Empty returning rands"))?;
        Ok(Self {
            seq,
            rand,
            raw_seqs: PyTuple::new(py, seqs).into_py(py),
            raw_rands: PyTuple::new(py, rands).into_py(py),
            time: datetime_from_ts(py, time)?.into_py(py),
        })
    }
}

pub struct PyHandler {
    queues: Py<PyList>,
    locals: TaskLocals,
    uin: i64,
}

impl PyHandler {
    pub fn new(queues: Py<PyList>, locals: TaskLocals, uin: i64) -> Self {
        Self {
            queues,
            locals,
            uin,
        }
    }
}

#[async_trait]
impl Handler for PyHandler {
    async fn handle(&self, event: QEvent) {
        let event_repr = format!("{event:?}");
        if let QEvent::ClientDisconnect(e) = event {
            match e.inner {
                DisconnectReason::Network => {
                    tracing::error!("网络错误, 尝试重连");
                }
                DisconnectReason::Actively(net) => match net {
                    NetworkStatus::Drop => {
                        tracing::error!("意料之外的内存释放");
                    }
                    NetworkStatus::NetworkOffline => {
                        tracing::error!("网络离线, 尝试重连");
                    }
                    NetworkStatus::KickedOffline => {
                        tracing::error!("其他设备登录, 被踢下线");
                    }
                    NetworkStatus::MsfOffline => {
                        tracing::error!("服务器强制下线");
                    }
                    _ => {}
                },
            }
            return;
        }
        let py_event = match self::converter::convert(event).await {
            Ok(obj) => obj,
            Err(e) => {
                tracing::error!("转换事件失败: {}", event_repr);
                py_use(|py| e.print_and_set_sys_last_vars(py));
                return;
            }
        };
        let mut handles: Vec<tokio::task::JoinHandle<Result<(), PyErr>>> = vec![];
        Python::with_gil(|py| {
            if py_event.as_ref(py).is_empty() {
                return;
            }
            let client = match py_client_refs(py).get_item(self.uin) {
                Ok(client) => client,
                Err(e) => {
                    tracing::error!("获取 client 引用失败: {}", event_repr);
                    e.print_and_set_sys_last_vars(py);
                    return;
                }
            };
            match py_event.as_ref(py).set_item("client", client) {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("设置 client 引用失败: {}", event_repr);
                    e.print_and_set_sys_last_vars(py);
                    return;
                }
            };
            let args: Py<PyTuple> = (py_event,).into_py(py);
            for q in self.queues.as_ref(py).iter().map(|q| q.into_py(py)) {
                let locals = self.locals.clone();
                let args = args.clone_ref(py);
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
                    tracing::error!("事件处理失败失败: {}", event_repr);
                    tracing::error!("Rust 无法收集回调结果: {:?}", err);
                }
                Ok(Err(err)) => {
                    tracing::error!("事件处理失败: {}", event_repr);
                    py_use(|py| err.print_and_set_sys_last_vars(py));
                }
                Ok(Ok(())) => {}
            };
        }
    }
}
