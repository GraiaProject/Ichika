#![feature(type_alias_impl_trait)]
#![feature(try_blocks)]
#![feature(concat_idents)]
#![feature(let_chains)]
#![feature(async_closure)]
#![feature(lint_reasons)]

use pyo3::prelude::*;
use pyo3_built::pyo3_built;

pub mod client;
mod events;
pub(crate) mod exc;
pub mod login;
mod loguru;
pub mod message;
mod utils;
type PyRet = PyResult<PyObject>;

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
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__build__", pyo3_built!(py, build_info))?;
    add_batch!(@fun m,
        loguru::getframe,
        message::elements::face_id_from_name,
        message::elements::face_name_from_id,
        login::password_login,
        login::qrcode_login
    );
    add_batch!(@cls m,
        client::PlumbingClient,
        client::friend::Friend,
        client::friend::FriendGroup,
        client::friend::FriendList,
        client::group::Group,
        client::group::Member,
        client::structs::AccountInfo,
        client::structs::OtherClientInfo,
        client::structs::RawMessageReceipt
    );
    register_event_module(py, m)?;
    loguru::init(m)?;
    Ok(())
}

fn register_event_module(py: Python, parent: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "ichika.core.events")?;
    add_batch!(@cls m,
        crate::events::GroupMessage,
        crate::events::TempMessage,
        crate::events::FriendMessage,
        crate::events::UnknownEvent
    );
    parent.add_submodule(m)?;
    parent.add("events", m)?;
    // See https://github.com/PyO3/pyo3/issues/759
    py.import("sys")?
        .getattr("modules")?
        .set_item("ichika.core.events", m)?;
    register_event_structs_module(py, m)?;
    Ok(())
}

fn register_event_structs_module(py: Python, parent: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "ichika.core.events.structs")?;
    add_batch!(@cls m,
        crate::events::structs::MessageSource,
        crate::events::structs::MemberInfo
    );
    parent.add_submodule(m)?;
    parent.add("structs", m)?;
    // See https://github.com/PyO3/pyo3/issues/759
    py.import("sys")?
        .getattr("modules")?
        .set_item("ichika.core.events.structs", m)?;
    Ok(())
}
