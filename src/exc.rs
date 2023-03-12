use pyo3::import_exception;
use pyo3::prelude::*;
use ricq::RQError;

import_exception!(ichika.exceptions, IchikaError);
import_exception!(ichika.exceptions, RICQError);
import_exception!(ichika.exceptions, LoginError);

// TODO: drop anyhow::Error

pub(crate) trait IntoPyErr {
    fn into_py(self) -> PyErr;
}

pub(crate) trait MapPyErr {
    type Output;
    fn py_res(self) -> Result<Self::Output, PyErr>;
}

impl IntoPyErr for RQError {
    fn into_py(self) -> PyErr {
        RICQError::new_err(format!("RICQ 出现错误: {self:?}"))
    }
}

impl<T, E> MapPyErr for Result<T, E>
where
    E: IntoPyErr,
{
    type Output = T;

    fn py_res(self) -> Result<Self::Output, PyErr> {
        match self {
            Ok(output) => Ok(output),
            Err(e) => Err(e.into_py()),
        }
    }
}
