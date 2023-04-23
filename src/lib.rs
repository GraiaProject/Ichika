#![feature(type_alias_impl_trait)]
#![feature(try_blocks)]
#![feature(concat_idents)]
#![feature(let_chains)]
#![feature(async_closure)]
#![feature(lint_reasons)]
#![feature(result_flattening)]
#![feature(iterator_try_collect)]

use pyo3::prelude::*;

mod build_info;
pub mod client;
mod events;
pub(crate) mod exc;
pub mod login;
mod loguru;
pub mod message;
mod utils;
type PyRet = PyResult<PyObject>;

macro_rules! add_batch {
    (@fun $m: ident, $($func: ty),*) => {
        $($m.add_function(wrap_pyfunction!($func, $m)?)?;)*
    };
    (@cls $m: ident, $($cls: ty),*) => {
        $($m.add_class::<$cls>()?;)*
    }
}

#[pymodule]
#[doc(hidden)]
pub fn core(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__build__", build_info::get_info(py)?)?;
    add_batch!(@fun m,
        loguru::getframe,
        message::elements::face_id_from_name,
        message::elements::face_name_from_id,
        login::password_login,
        login::qrcode_login
    );
    add_batch!(@cls m,
        client::PlumbingClient,
        client::friend::Friend,
        client::friend::FriendGroup,
        client::friend::FriendList,
        client::group::Group,
        client::group::Member,
        client::structs::AccountInfo,
        client::structs::OtherClientInfo,
        client::structs::RawMessageReceipt,
        client::structs::OCRResult,
        client::structs::OCRText,
        events::MessageSource
    );
    loguru::init(m)?;
    Ok(())
}
