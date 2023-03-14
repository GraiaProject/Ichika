use anyhow::Result;
use futures_util::Future;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

// use pyo3::types::*;

/// 获取 Python 的 None。
pub fn py_none() -> PyObject {
    Python::with_gil(|py| py.None())
}

pub trait AsPython {
    fn obj(self) -> PyObject;
}

impl<T> AsPython for T
where
    T: IntoPy<PyObject>,
{
    fn obj(self) -> PyObject {
        Python::with_gil(|py| self.into_py(py))
    }
}

pub fn py_bytes(data: &[u8]) -> Py<PyBytes> {
    py_use(|py| PyBytes::new(py, data).into_py(py))
}

/// 构造一个 Python 的 dict。
#[macro_export]
#[doc(hidden)]
macro_rules! py_dict {
    ($py:expr, $($name:expr => $value:expr),*) => {
        {
            let dict = ::pyo3::types::PyDict::new($py);
            $(
                dict.set_item($name, $value).expect("Failed to set_item on dict");
            )*
            dict
        }
    };
}

/// 等价于 `Some(py_dict!(..))`，用于指定 kwargs。
#[macro_export]
#[doc(hidden)]
macro_rules! kwargs {
    ($py:expr, $($name:expr => $value:expr),*) => {
        Some($crate::py_dict!($py, $($name => $value),*))
    };
}

#[macro_export]
macro_rules! import_call {
    ($py:expr, $module:expr => $attr:expr => $arg:expr) => {
        $py.import(::pyo3::intern!($py, $module))?
            .getattr(::pyo3::intern!($py, $attr))?
            .call1(($arg,))
    };
    ($py:expr, $module:expr => $attr:expr => @tuple $arg:expr) => {
        $py.import(::pyo3::intern!($py, $module))?
            .getattr(::pyo3::intern!($py, $attr))?
            .call1($arg)
    };
}

#[macro_export]
macro_rules! props {
    ($self_t:ident @ $cls:ident : $($name:ident => [$type:ty] $res:stmt);* ;) => {
        #[::pyo3::pymethods]
        impl $cls {
            $(
                #[getter]
                pub fn $name(&$self_t) -> $type {
                    $res
                }
            )*
        }
    };
}

/// 将 [`tokio`] 的 Future 包装为 Python 的 Future。
pub fn py_future<F, T>(py: Python, future: F) -> PyResult<&PyAny>
where
    F: Future<Output = Result<T, anyhow::Error>> + Send + 'static,
    T: IntoPy<PyObject>,
{
    pyo3_asyncio::tokio::future_into_py(py, async move { Ok(future.await?) })
}

/// 自动重试直到得到 `Ok(..)`。
pub async fn py_retry<F, T, D>(
    mut max_count: usize,
    mut f: impl FnMut() -> F,
    mut on_retry: impl FnMut(PyErr, usize) -> D,
) -> PyResult<T>
where
    F: Future<Output = PyResult<T>>,
    D: Future<Output = ()>,
{
    loop {
        match f().await {
            Ok(t) => return Ok(t),
            Err(e) => {
                if max_count == 0 {
                    return Err(e);
                }
                max_count -= 1;
                on_retry(e, max_count).await;
            }
        }
    }
}

pub fn py_try<F, R>(f: F) -> PyResult<R>
where
    F: for<'py> FnOnce(Python<'py>) -> PyResult<R>,
{
    Python::with_gil(f)
}

pub fn py_use<F, R>(f: F) -> R
where
    F: for<'py> FnOnce(Python<'py>) -> R,
{
    Python::with_gil(f)
}

#[macro_export]
macro_rules! static_py_fn {
    ($name:ident, $cell_name:ident, $module:expr, [$($attr:expr),*]) => {
        #[allow(non_upper_case_globals, reason = "Not controllable via declarative macros")]
        static $cell_name: ::pyo3::once_cell::GILOnceCell<PyObject> = ::pyo3::once_cell::GILOnceCell::new();

        pub fn $name(python: ::pyo3::marker::Python) -> &pyo3::PyAny {
            $cell_name.get_or_init(python, || {
                python
                .import(::pyo3::intern!(python, $module)).expect(concat!("Unable to import module ", $module))
                $(.getattr(::pyo3::intern!(python, $attr)).expect(concat!("Unable to get attribute ", $attr)))*
                .into()
                }
            )
            .as_ref(python)
        }
    };
}

#[macro_export]
macro_rules! call_static_py {
    ($pth:expr, $py:expr, ($($arg:expr),*)) => {
        $pth($py).call1(
            ($($arg),*)
        )
    };
    ($pth:expr, $py:expr, ($($arg:expr),*) ! $reason:expr) => {
        $pth($py).call1(
            ($($arg,)*)
        )
        .expect($reason)
        .into()
    }
}

static_py_fn!(
    datetime_from_ts,
    __DT_CELL,
    "datetime",
    ["datetime", "fromtimestamp"]
);

static_py_fn!(partial, __PARTIAL_CELL, "functools", ["partial"]);
