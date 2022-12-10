use pyo3::{prelude::*, types::*};

#[pyclass(module = "ichika.client.structs#rs")]
#[derive(Debug, Clone)]
pub struct AccountInfo {
    #[pyo3(get)]
    pub nickname: Py<PyString>,
    #[pyo3(get)]
    pub age: u8,
    #[pyo3(get)]
    pub gender: u8,
}

crate::repr!(AccountInfo);

#[pyclass(module = "ichika.client.structs#rs")]
#[derive(Debug, Clone)]
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

crate::repr!(OtherClientInfo);

#[pyclass(module = "ichika.client.structs#rs")]
#[derive(Debug, Clone)]
pub struct RawMessageReceipt {
    #[pyo3(get)]
    pub seqs: Py<PyTuple>,
    #[pyo3(get)]
    pub rands: Py<PyTuple>,
    #[pyo3(get)]
    pub time: i64,
}
crate::repr!(RawMessageReceipt);
