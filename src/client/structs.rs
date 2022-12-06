use pyo3::prelude::*;

#[pyclass(module = "ichika.client.structs#rs")]
pub struct AccountInfo {
    #[pyo3(get)]
    pub nickname: String,
    #[pyo3(get)]
    pub age: u8,
    #[pyo3(get)]
    pub gender: u8,
}
