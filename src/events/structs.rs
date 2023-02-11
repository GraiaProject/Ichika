use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3_repr::PyRepr;

use crate::call_static_py;
use crate::utils::datetime_from_ts;
#[pyclass]
#[derive(PyRepr, Clone)]
pub struct MessageSource {
    #[pyo3(get)]
    pub seqs: Py<PyTuple>,
    #[pyo3(get)]
    pub rands: Py<PyTuple>,
    #[pyo3(get)]
    pub time: PyObject,
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

#[pyclass]
#[derive(PyRepr, Clone)]
pub struct FriendInfo {
    #[pyo3(get)]
    pub uin: i64,
    #[pyo3(get)]
    pub nickname: String,
}
