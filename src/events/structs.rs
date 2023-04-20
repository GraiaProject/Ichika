use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3_repr::PyRepr;

use crate::utils::datetime_from_ts;
#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct MessageSource {
    pub seq: i32,
    pub rand: i32,
    pub raw_seqs: Py<PyTuple>,
    pub raw_rands: Py<PyTuple>,
    pub time: PyObject,
}

impl MessageSource {
    pub fn new(py: Python, seqs: &[i32], rands: &[i32], time: i32) -> PyResult<Self> {
        let seq = *seqs
            .first()
            .ok_or_else(|| PyIndexError::new_err("Empty returning rands"))?;
        let rand = *rands
            .first()
            .ok_or_else(|| PyIndexError::new_err("Empty returning rands"))?;
        Ok(Self {
            seq,
            rand,
            raw_seqs: PyTuple::new(py, seqs).into_py(py),
            raw_rands: PyTuple::new(py, rands).into_py(py),
            time: datetime_from_ts(py, time)?.into_py(py),
        })
    }
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct FriendInfo {
    pub uin: i64,
    pub nickname: String,
}
