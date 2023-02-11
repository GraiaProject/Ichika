//! 消息元素。

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use ricq::msg::elem::{FriendImage, GroupImage, MarketFace};

use crate::props;

#[pyfunction]
pub fn face_name_from_id(id: i32) -> String {
    ricq_core::msg::elem::Face::name(id).to_owned()
}

#[pyfunction]
pub fn face_id_from_name(name: String) -> Option<i32> {
    match ricq_core::msg::elem::Face::new_from_name(&name) {
        Some(f) => Some(f.index),
        None => None,
    }
}

macro_rules! py_seal {
    ($name:ident => $type:ty) => {
        #[pyclass(module = "ichika.message.elements#rs.inner")]
        #[derive(::pyo3_repr::PyRepr, Clone)]
        pub struct $name {
            pub inner: $type,
        }
    };
}

py_seal!(SealedMarketFace => MarketFace);

#[pymethods]
impl SealedMarketFace {
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }
}

py_seal!(SealedGroupImage => GroupImage);
py_seal!(SealedFriendImage => FriendImage);

props!(self @ SealedGroupImage:
    md5 => [Py<PyBytes>] Python::with_gil(|py| PyBytes::new(py, &self.inner.md5).into_py(py));
    size => [u32] self.inner.size;
    width => [u32] self.inner.width;
    height => [u32] self.inner.height;
    image_type => [i32] self.inner.image_type;
);

props!(self @ SealedFriendImage:
    md5 => [Py<PyBytes>] Python::with_gil(|py| PyBytes::new(py, &self.inner.md5).into_py(py));
    size => [u32] self.inner.size;
    width => [u32] self.inner.width;
    height => [u32] self.inner.height;
    image_type => [i32] self.inner.image_type;
);
