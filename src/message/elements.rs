//! 消息元素。

use pyo3::prelude::*;
use ricq::msg::elem::MarketFace;

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

#[pyclass(module = "ichika.message.elements#rs.inner")]
#[derive(Debug, Clone)]
pub struct MarketFaceImpl {
    pub face: MarketFace,
}

#[pymethods]
impl MarketFaceImpl {
    #[getter]
    fn name(&self) -> String {
        self.face.name.clone()
    }
}
