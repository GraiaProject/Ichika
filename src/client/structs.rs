use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_repr::PyRepr;
#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct AccountInfo {
    pub nickname: String,
    pub age: u8,
    pub gender: u8,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct OtherClientInfo {
    pub app_id: i64,
    pub instance_id: i32,
    pub sub_platform: String,
    pub device_kind: String,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct RawMessageReceipt {
    pub seqs: Py<PyTuple>,
    pub rands: Py<PyTuple>,
    pub time: i64,
    pub kind: String,
    pub target: i64,
}
