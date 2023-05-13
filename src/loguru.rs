//! [`tracing`] 与 Python 的 Loguru 的桥接模块。

use std::fmt::Write;
use std::sync::Arc;

use pyo3::exceptions::PyRuntimeError;
use pyo3::intern;
use pyo3::once_cell::GILOnceCell;
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_repr::PyRepr;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

use crate::py_dict;

/// 初始化日志输出。
pub(crate) fn init(module: &PyModule) -> PyResult<()> {
    // 输出桥接
    let layer = LoguruLayer::new()?;
    tracing_subscriber::registry()
        .with(layer)
        .with(
            // 筛选不同包的日志级别
            tracing_subscriber::filter::Targets::new()
                .with_target("ricq", Level::DEBUG)
                .with_target("ichika", Level::DEBUG),
        )
        .init();
    // 注入 getframe
    Python::with_gil(|py| -> PyResult<()> {
        let logger_module = py.import("loguru")?.getattr("_logger")?;
        logger_module.setattr("get_frame", module.getattr("_getframe")?)
    })?;
    Ok(())
}

/// 将 [`tracing`] 的输出桥接到 Python 的 Loguru 中。
pub(crate) struct LoguruLayer {
    log_fn: PyObject,
}

impl LoguruLayer {
    /// 创建一个新的 `LoguruLayer` 对象。
    pub(crate) fn new() -> PyResult<Self> {
        let log_fn = Python::with_gil(|py| -> PyResult<PyObject> {
            let loguru = py.import("loguru")?;
            let logger = loguru.getattr("logger")?;
            let log_fn = logger.getattr("log")?;
            Ok(log_fn.into())
        })?;
        Ok(LoguruLayer { log_fn })
    }
}

impl<S> Layer<S> for LoguruLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        // 记录日志发生的位置，保存为伪 Python 堆栈
        Python::with_gil(|py| {
            if let Ok(mut frame) = LAST_RUST_FRAME
                .get_or_init(py, || Arc::new(std::sync::RwLock::new(None)))
                .write()
            {
                let meta = event.metadata();

                *frame = FakePyFrame::new(
                    &meta
                        .module_path()
                        .unwrap_or_else(|| event.metadata().target())
                        .split("::")
                        .collect::<Vec<_>>()
                        .join("."),
                    meta.file().unwrap_or("<rust>"),
                    "<rust>",
                    meta.line().unwrap_or(0),
                )
                .ok();
            }
        });

        let message = {
            let mut visiter = LoguruVisiter::new();
            event.record(&mut visiter);
            visiter.0
        };
        let level = match event.metadata().level().as_str() {
            "WARN" => "WARNING", // 处理两个级别名称不一致的问题
            s => s,
        };
        Python::with_gil(|py| {
            let level: Py<PyString> = level.into_py(py);
            let message: PyObject = message.into_py(py);
            self.log_fn.call1(py, (level, message)).unwrap();
        });
    }
}

/// 遍历并格式化日志信息。
struct LoguruVisiter(String);

impl LoguruVisiter {
    /// 创建一个新的 `LoguruVisiter` 对象。
    pub fn new() -> Self {
        LoguruVisiter(String::new())
    }
}

impl tracing::field::Visit for LoguruVisiter {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0.push_str(value);
        } else {
            write!(self.0, "{}={value}", field.name()).unwrap();
        }
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        write!(self.0, "{}={value}", field.name()).unwrap();
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            write!(self.0, "{value:?}").unwrap();
        } else {
            write!(self.0, "{}={value:?}", field.name()).unwrap();
        }
    }
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct FakePyFrame {
    f_globals: Py<PyDict>,
    f_code: Py<FakePyCode>,
    f_lineno: u32,
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct FakePyCode {
    co_filename: Py<PyString>,
    co_name: Py<PyString>,
}

impl FakePyFrame {
    fn new(name: &str, file_path: &str, function: &str, line: u32) -> PyResult<FakePyFrame> {
        let f_globals = Python::with_gil(|py| {
            let name: Py<PyString> = name.into_py(py);
            py_dict!(py, "__name__" => name).into()
        });
        let f_code = Python::with_gil(|py| {
            Py::new(
                py,
                FakePyCode {
                    co_filename: PyString::new(py, file_path).into(),
                    co_name: PyString::new(py, function).into(),
                },
            )
        })?;
        Ok(FakePyFrame {
            f_globals,
            f_code,
            f_lineno: line,
        })
    }
}

#[pyfunction]
#[pyo3(name = "_getframe")]
#[doc(hidden)]
pub fn getframe(py: Python, depth: usize) -> PyResult<FakePyFrame> {
    let frames: &PyList = py
        .import("inspect")?
        .call_method("stack", (), None)?
        .extract()?;
    Ok(if frames.len() > depth {
        let frame_info = frames.get_item(depth)?;
        let name = frame_info
            .getattr(intern!(py, "frame"))?
            .getattr(intern!(py, "f_globals"))?
            .get_item(intern!(py, "__name__"))?
            .extract()?;
        let file_path = frame_info.getattr(intern!(py, "filename"))?.extract()?;
        let function = frame_info.getattr(intern!(py, "function"))?.extract()?;
        let line = frame_info.getattr(intern!(py, "lineno"))?.extract()?;
        FakePyFrame::new(name, file_path, function, line)?
    } else {
        let frame = LAST_RUST_FRAME
            .get_or_init(py, || Arc::new(std::sync::RwLock::new(None)))
            .read()
            .map(|frame| {
                frame.as_ref().map_or_else(
                    || FakePyFrame::new("<unknown>", "", "", 0),
                    |f| Ok(f.clone()),
                )
            })
            .map_err(|e| PyRuntimeError::new_err(format!("Unable to create Rust frame: {e:?}")));
        frame??
    })
}

/// 最后一次日志记录时的 rust 堆栈
static LAST_RUST_FRAME: GILOnceCell<Arc<std::sync::RwLock<Option<FakePyFrame>>>> =
    GILOnceCell::new();
