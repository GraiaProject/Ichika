//! 消息元素。

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use ricq::msg::elem::{FriendImage, GroupImage, MarketFace};

use crate::props;
use crate::utils::py_bytes;

#[pyfunction]
pub fn face_name_from_id(id: i32) -> String {
    ricq_core::msg::elem::Face::name(id).to_owned()
}

#[pyfunction]
pub fn face_id_from_name(name: &str) -> Option<i32> {
    match ricq_core::msg::elem::Face::new_from_name(name) {
        Some(f) => Some(f.index),
        None => None,
    }
}

macro_rules! py_seal {
    ($name:ident => $type:ty) => {
        #[::pyo3::pyclass]
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
    md5 => [Py<PyBytes>] py_bytes(&self.inner.md5);
    size => [u32] self.inner.size;
    width => [u32] self.inner.width;
    height => [u32] self.inner.height;
    image_type => [i32] self.inner.image_type;
);

props!(self @ SealedFriendImage:
    md5 => [Py<PyBytes>] py_bytes(&self.inner.md5);
    size => [u32] self.inner.size;
    width => [u32] self.inner.width;
    height => [u32] self.inner.height;
    image_type => [i32] self.inner.image_type;
);

py_seal!(SealedAudio => ricq_core::pb::msg::Ptt);

props!(self @ SealedAudio:
    md5 => [Py<PyBytes>] py_bytes(self.inner.file_md5());
    size => [i32] self.inner.file_size();
    file_type => [i32] self.inner.file_type();
);
