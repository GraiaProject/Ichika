use pyo3::prelude::*;

#[pyclass(module = "ichika.client.structs#rs")]
#[derive(Debug, Clone)]
pub struct AccountInfo {
    #[pyo3(get)]
    pub nickname: String,
    #[pyo3(get)]
    pub age: u8,
    #[pyo3(get)]
    pub gender: u8,
}

#[pymethods]
impl AccountInfo {
    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

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

#[pymethods]
impl OtherClientInfo {
    pub fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
