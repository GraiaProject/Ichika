use async_trait::async_trait;
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_asyncio::{into_future_with_locals, TaskLocals};
use pyo3_repr::PyRepr;
use ricq::handler::{Handler, QEvent};

pub mod converter;
pub mod structs;
use structs::MessageSource;

use self::structs::FriendInfo;
use crate::client::group::{Group, Member};
use crate::utils::{py_try, py_use};

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
    group: Group,
    sender: Member,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct GroupRecallMessage {
    time: PyObject, // PyDatetime
    group: Group,
    author: Member,
    operator: Member,
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
    group: Group,
    sender: Member,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct GroupNudge {
    group: Group,
    sender: Member,
    receiver: Member,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct FriendNudge {
    sender: FriendInfo,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct NewFriend {
    friend: FriendInfo,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct NewMember {
    group: Group,
    member: Member,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct MemberLeaveGroup {
    group_uin: i64,
    member_uin: i64,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct GroupDisband {
    group_uin: i64,
    operator_uin: i64,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct FriendDeleted {
    friend_uin: i64,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct MemberMute {
    group: Group,
    target: Member,
    operator: Member,
    duration: PyObject, // datetime.timedelta | Literal[False]
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct GroupMute {
    group: Group,
    operator: Member,
    status: bool,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct MemberPermissionChange {
    group: Group,
    target: Member,
    permission: u8,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct GroupInfoUpdate {
    group: Group,
    operator: Member,
    info: Py<PyDict>, // GroupInfo
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct NewFriendRequest {
    seq: i64,
    uin: i64,
    nickname: String,
    message: String,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct JoinGroupRequest {
    seq: i64,
    time: PyObject,
    group_uin: i64,
    group_name: String,
    request_uin: i64,
    request_nickname: String,
    suspicious: bool,
    invitor_uin: Option<i64>,
    invitor_nickname: Option<String>,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct JoinGroupInvitation {
    seq: i64,
    time: PyObject,
    group_uin: i64,
    group_name: String,
    invitor_uin: i64,
    invitor_nickname: String,
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
                tracing::error!("转换事件失败: {}", event_repr);
                py_use(|py| e.print_and_set_sys_last_vars(py));
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
