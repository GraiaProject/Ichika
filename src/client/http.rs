use std::collections::HashMap;

use async_trait::async_trait;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3_asyncio::{into_future_with_locals, TaskLocals};
use ricq::ext::http::{HttpClient as RQHttpClient, HttpMethod as RQHttpMethod};
use ricq::RQError;

use crate::utils::py_try;

pub fn get_rust_client<'py>(py: Python<'py>, callable: &'py PyAny) -> PyResult<PyHttpClient> {
    let locals = TaskLocals::with_running_loop(py)?.copy_context(py)?;
    Ok(PyHttpClient {
        callable: callable.into_py(py),
        locals,
    })
}

pub struct PyHttpClient {
    callable: PyObject,
    locals: TaskLocals,
}

fn http_method_to_string(method: RQHttpMethod) -> String {
    match method {
        RQHttpMethod::GET => "get".into(),
        RQHttpMethod::POST => "post".into(),
    }
}

#[async_trait]
impl RQHttpClient for PyHttpClient {
    async fn make_request(
        &mut self,
        method: RQHttpMethod,
        url: String,
        header: &HashMap<String, String>,
        body: bytes::Bytes,
    ) -> Result<bytes::Bytes, RQError> {
        let py_res = py_try(|py| {
            let header = header.clone().into_py(py);
            let body = PyBytes::new(py, &body);
            into_future_with_locals(
                &self.locals,
                self.callable.as_ref(py).call1((
                    http_method_to_string(method),
                    url,
                    header,
                    body,
                ))?,
            )
        })
        .map_err(|e| RQError::Other(e.to_string()))?
        .await
        .map_err(|e| RQError::Other(e.to_string()))?;
        py_try(move |py| {
            let bin = py_res.as_ref(py).downcast::<PyBytes>()?;
            Ok(bytes::Bytes::from(Vec::from(bin.as_bytes())))
        })
        .map_err(|e| RQError::Decode(e.to_string()))
    }
}
