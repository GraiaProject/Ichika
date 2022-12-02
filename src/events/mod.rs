mod structs;
use async_trait::async_trait;
use pyo3::types::PyList;
use pyo3::{prelude::*, types::PyTuple};
use ricq::client::handler::Handler;
use ricq::client::handler::QEvent;
pub use structs::*;

macro_rules! mk_convert {
    ($($name: ident),*) => {
        fn convert(e: QEvent, py: Python) -> Py<PyAny> {
            match e{
                $(QEvent::$name(inner) => $name::from(inner).into_py(py)),*
            }
        }
    };
}

mk_convert!(
    Login,
    GroupMessage,
    GroupAudioMessage,
    FriendMessage,
    FriendAudioMessage,
    GroupTempMessage,
    GroupRequest,
    SelfInvited,
    NewFriendRequest,
    NewMember,
    GroupMute,
    FriendMessageRecall,
    NewFriend,
    GroupMessageRecall,
    GroupLeave,
    GroupDisband,
    FriendPoke,
    GroupNameUpdate,
    DeleteFriend,
    MemberPermissionChange,
    KickedOffline,
    MSFOffline
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
    async fn handle(&self, e: QEvent) {
        Python::with_gil(|py| {
            let py_event = convert(e, py);
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
