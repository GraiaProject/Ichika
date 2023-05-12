use std::backtrace::Backtrace;

use pyo3::import_exception;
use pyo3::prelude::*;
use ricq::RQError;

use crate::utils::py_use;

import_exception!(ichika.exceptions, IchikaError);
import_exception!(ichika.exceptions, RICQError);
import_exception!(ichika.exceptions, LoginError);

#[derive(Debug)]
enum InnerError {
    RQ(RQError),
    Python(PyErr),
    Other(Box<dyn std::error::Error>),
}

#[derive(Debug)]
pub struct Error {
    inner: InnerError,
    backtrace: Backtrace,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self {
            inner: InnerError::RQ(RQError::IO(value)),
            backtrace: Backtrace::force_capture(),
        }
    }
}

impl From<RQError> for Error {
    fn from(value: RQError) -> Self {
        Self {
            inner: InnerError::RQ(value),
            backtrace: Backtrace::force_capture(),
        }
    }
}

impl From<PyErr> for Error {
    fn from(value: PyErr) -> Self {
        Self {
            inner: InnerError::Python(value),
            backtrace: Backtrace::force_capture(),
        }
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self {
            inner: InnerError::Other(value),
            backtrace: Backtrace::force_capture(),
        }
    }
}

impl IntoPy<PyErr> for Error {
    fn into_py(self, _: Python) -> PyErr {
        let bt = self.backtrace;
        match self.inner {
            InnerError::RQ(e) => RICQError::new_err(format!("RICQ 发生错误: {e:?}\n{bt}")),
            InnerError::Python(e) => e,
            InnerError::Other(e) => IchikaError::new_err(format!("未知错误: {e:?}\n{bt}")),
        }
    }
}

impl From<Error> for PyErr {
    fn from(value: Error) -> Self {
        py_use(|py| value.into_py(py))
    }
}

pub(crate) trait MapPyErr {
    type Output;
    fn py_res(self) -> Result<Self::Output, PyErr>;
}

impl<T, E> MapPyErr for Result<T, E>
where
    E: Into<Error>,
{
    type Output = T;

    fn py_res(self) -> Result<Self::Output, PyErr> {
        match self {
            Ok(output) => Ok(output),
            Err(e) => Err({
                let e: Error = e.into();
                e.into()
            }),
        }
    }
}

pub type IckResult<T> = Result<T, Error>;
