use pyo3::{prelude::*, types::*};
use pyo3_repr::PyRepr;
#[pyclass(module = "ichika.client.structs#rs")]
#[derive(PyRepr, Clone)]
pub struct AccountInfo {
    #[pyo3(get)]
    pub nickname: Py<PyString>,
    #[pyo3(get)]
    pub age: u8,
    #[pyo3(get)]
    pub gender: u8,
}

#[pyclass(module = "ichika.client.structs#rs")]
#[derive(PyRepr, Clone)]
pub struct OtherClientInfo {
    #[pyo3(get)]
    pub app_id: i64,
    #[pyo3(get)]
    pub instance_id: i32,
    #[pyo3(get)]
    pub sub_platform: String,
    #[pyo3(get)]
    pub device_kind: String,
}

#[pyclass(module = "ichika.client.structs#rs")]
#[derive(PyRepr, Clone)]
pub struct RawMessageReceipt {
    #[pyo3(get)]
    pub seqs: Py<PyTuple>,
    #[pyo3(get)]
    pub rands: Py<PyTuple>,
    #[pyo3(get)]
    pub time: i64,
}
