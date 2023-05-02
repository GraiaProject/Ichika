use pyo3::{prelude::*, types::PyTuple};

use crate::repr;

#[pyclass]
#[derive(Debug, Clone)]
pub struct MessageSource {
    #[pyo3(get)]
    pub seqs: Py<PyTuple>,
    #[pyo3(get)]
    pub rands: Py<PyTuple>,
}

impl MessageSource {
    pub fn new(py: Python<'_>, seqs: &[i32], rands: &[i32]) -> Self {
        Self {
            seqs: PyTuple::new(py, seqs).into_py(py),
            rands: PyTuple::new(py, rands).into_py(py),
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct GroupInfo {
    #[pyo3(get)]
    pub uin: i64,
    #[pyo3(get)]
    pub name: String,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct MemberInfo {
    #[pyo3(get)]
    pub uin: i64,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub group: GroupInfo,
    #[pyo3(get)]
    pub permission: u8,
}

repr!(MessageSource, GroupInfo, MemberInfo);
