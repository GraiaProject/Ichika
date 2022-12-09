#![feature(type_alias_impl_trait)]

use pyo3::prelude::*;
use pyo3_built::pyo3_built;

pub mod client;
mod device;
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

#[pymodule]
#[doc(hidden)]
pub fn core(py: Python, m: &PyModule) -> PyResult<()> {
    // 初始化
    m.add_function(wrap_pyfunction!(init_log, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__build__", pyo3_built!(py, build_info))?;
    m.add_function(wrap_pyfunction!(loguru::getframe, m)?)?;
    m.add_function(wrap_pyfunction!(message::elements::face_id_from_name, m)?)?;
    m.add_function(wrap_pyfunction!(message::elements::face_name_from_id, m)?)?;
    m.add_class::<login::Account>()?;
    m.add_class::<client::plumbing::PlumbingClient>()?;
    Ok(())
}
