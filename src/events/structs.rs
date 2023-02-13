use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3_repr::PyRepr;

use crate::call_static_py;
use crate::utils::datetime_from_ts;
#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct MessageSource {
    pub seqs: Py<PyTuple>,
    pub rands: Py<PyTuple>,
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

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct GroupInfo {
    pub uin: i64,
    pub name: String,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct MemberInfo {
    pub uin: i64,
    pub name: String,
    pub group: GroupInfo,
    pub permission: u8,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct FriendInfo {
    pub uin: i64,
    pub nickname: String,
}
