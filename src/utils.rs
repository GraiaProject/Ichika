use anyhow::Result;
use futures_util::Future;
use pyo3::prelude::*;

// use pyo3::types::*;

/// 获取 Python 的 None。
pub fn py_none() -> PyObject {
    Python::with_gil(|py| py.None())
}

/// 将 Rust 定义的 Python 类实例化。
pub fn py_obj<T>(obj: impl Into<PyClassInitializer<T>>) -> PyResult<Py<T>>
where
    T: pyo3::PyClass,
{
    Python::with_gil(|py| Py::new(py, obj))
}

// pub fn py_list<T, C, U>(list: impl IntoIterator<Item = T, IntoIter = U>) -> PyResult<Py<PyList>>
// where
//     T: Into<PyClassInitializer<C>>,
//     C: pyo3::PyClass,
//     U: ExactSizeIterator<Item = T>,
// {
//     Python::with_gil(|py| {
//         Ok(PyList::new(
//             py,
//             list.into_iter()
//                 .map(|item| PyCell::new(py, item))
//                 .collect::<Result<Vec<_>, _>>()?,
//         )
//         .into())
//     })
// }

/// 构造一个 Python 的 dict。
#[macro_export]
#[doc(hidden)]
macro_rules! py_dict {
    ($py:expr, $($name:expr => $value:expr),*) => {
        {
            let dict = PyDict::new($py);
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

/// 创建 Python 字符串（有缓存）。
#[macro_export]
#[doc(hidden)]
macro_rules! py_intern {
    ($s:expr) => {
        Python::with_gil(|py| ::pyo3::types::PyString::intern(py, $s).into_py(py))
    };
}

/// 创建 Python 字符串（无缓存）。
#[macro_export]
#[doc(hidden)]
macro_rules! py_str {
    ($s:expr) => {
        Python::with_gil(|py| ::pyo3::types::PyString::new(py, $s).into_py(py))
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
pub async fn retry<F, T, D>(
    mut max_count: usize,
    mut f: impl FnMut() -> F,
    mut on_retry: impl FnMut(anyhow::Error, usize) -> D,
) -> Result<T>
where
    F: Future<Output = Result<T>>,
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
