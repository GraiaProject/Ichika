//! 消息元素。

use pyo3::prelude::*;
use ricq::msg::elem::{FriendImage, GroupImage, MarketFace};

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
    ($name: ident => $type: ty) => {
        #[pyclass(module = "ichika.message.elements#rs.inner")]
        #[derive(Debug, Clone)]
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

#[pymethods]
impl SealedGroupImage {
    #[getter]
    fn md5(&self) -> Vec<u8> {
        self.inner.md5.clone()
    }
    #[getter]
    fn size(&self) -> u32 {
        self.inner.size
    }
    #[getter]
    fn width(&self) -> u32 {
        self.inner.width
    }
    #[getter]
    fn height(&self) -> u32 {
        self.inner.height
    }
    #[getter]
    fn image_type(&self) -> i32 {
        self.inner.image_type
    }
}

#[pymethods]
impl SealedFriendImage {
    #[getter]
    fn md5(&self) -> Vec<u8> {
        self.inner.md5.clone()
    }
    #[getter]
    fn size(&self) -> u32 {
        self.inner.size
    }
    #[getter]
    fn width(&self) -> u32 {
        self.inner.width
    }
    #[getter]
    fn height(&self) -> u32 {
        self.inner.height
    }
    #[getter]
    fn image_type(&self) -> i32 {
        self.inner.image_type
    }
}
