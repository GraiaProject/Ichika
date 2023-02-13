#![feature(type_alias_impl_trait)]
#![feature(try_blocks)]
#![feature(concat_idents)]

use pyo3::prelude::*;
use pyo3_built::pyo3_built;
use ricq::RQError;

pub mod client;
mod events;
pub mod login;
mod loguru;
pub mod message;
mod utils;

type PyRet = PyResult<PyObject>;

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
        message::elements::face_name_from_id
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

fn register_event_structs_module(py: Python<'_>, parent: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "ichika.core.events.structs")?;
    add_batch!(@cls m,
        crate::events::structs::MessageSource,
        crate::events::structs::MemberInfo,
        crate::events::structs::GroupInfo
    );
    parent.add_submodule(m)?;
    parent.add("structs", m)?;
    // See https://github.com/PyO3/pyo3/issues/759
    py.import("sys")?
        .getattr("modules")?
        .set_item("ichika.core.events.structs", m)?;
    Ok(())
}

pub struct RICQError(RQError);

impl From<RICQError> for PyErr {
    fn from(error: RICQError) -> Self {
        pyo3::exceptions::PyRuntimeError::new_err(format!("RICQError: {}", error.0.to_string()))
    }
}

impl From<RQError> for RICQError {
    fn from(other: RQError) -> Self {
        Self(other)
    }
}
