use pyo3::{prelude::*, types::PyTuple};

use crate::{call_static_py, utils::datetime_from_ts};
use pyo3_repr::PyRepr;
#[pyclass]
#[derive(PyRepr, Clone)]
pub struct MessageSource {
    #[pyo3(get)]
    pub seqs: Py<PyTuple>,
    #[pyo3(get)]
    pub rands: Py<PyTuple>,
    #[pyo3(get)]
    pub time: Py<PyAny>,
}

impl MessageSource {
    pub fn new(py: Python<'_>, seqs: &[i32], rands: &[i32], time: i32) -> Self {
        Self {
            seqs: PyTuple::new(py, seqs).into_py(py),
            rands: PyTuple::new(py, rands).into_py(py),
            time: call_static_py!(datetime_from_ts, py, (time)! "Unable to convert time"),
        }
    }
}

#[pyclass]
#[derive(PyRepr, Clone)]
pub struct GroupInfo {
    #[pyo3(get)]
    pub uin: i64,
    #[pyo3(get)]
    pub name: String,
}

#[pyclass]
#[derive(PyRepr, Clone)]
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
