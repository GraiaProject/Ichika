#![feature(type_alias_impl_trait)]

use pyo3::prelude::*;
use pyo3_built::pyo3_built;

pub mod client;
mod events;
pub mod login;
mod loguru;
pub mod message;
mod utils;

#[pyfunction]
pub fn init_log(module: &PyModule) -> PyResult<()> {
    // 设置日志输出
    loguru::init(module)?;
    Ok(())
}

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/build-info.rs"));
}

macro_rules! add_batch {
    (@fun $m: ident, $($func: ty),*) => {
        $($m.add_function(wrap_pyfunction!($func, $m)?)?;)*
    };
    (@cls $m: ident, $($cls: ty),*) => {
        $($m.add_class::<$cls>()?;)*
    }
}

#[pymodule]
#[doc(hidden)]
pub fn core(py: Python, m: &PyModule) -> PyResult<()> {
    // 初始化
    m.add_function(wrap_pyfunction!(init_log, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__build__", pyo3_built!(py, build_info))?;
    add_batch!(@fun m,
        loguru::getframe,
        message::elements::face_id_from_name,
        message::elements::face_name_from_id,
        message::convert::preview_raw_chain
    );
    add_batch!(@cls m,
        login::Account,
        client::PlumbingClient
    );
    register_event_module(py, m)?;
    Ok(())
}

fn register_event_module(py: Python<'_>, parent: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "ichika.core.events")?;
    add_batch!(@cls m,
        crate::events::Login,
        crate::events::GroupMessage,
        crate::events::GroupAudioMessage,
        crate::events::FriendMessage,
        crate::events::FriendAudioMessage,
        crate::events::GroupTempMessage,
        crate::events::GroupRequest,
        crate::events::SelfInvited,
        crate::events::NewFriendRequest,
        crate::events::NewMember,
        crate::events::GroupMute,
        crate::events::FriendMessageRecall,
        crate::events::NewFriend,
        crate::events::GroupMessageRecall,
        crate::events::GroupLeave,
        crate::events::GroupDisband,
        crate::events::FriendPoke,
        crate::events::GroupNameUpdate,
        crate::events::DeleteFriend,
        crate::events::MemberPermissionChange,
        crate::events::KickedOffline,
        crate::events::MSFOffline
    );
    parent.add_submodule(m)?;
    // See https://github.com/PyO3/pyo3/issues/759
    py.import("sys")?
        .getattr("modules")?
        .set_item("ichika.core.events", m)?;
    Ok(())
}
