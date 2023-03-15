use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_repr::PyRepr;
#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct AccountInfo {
    pub nickname: String,
    pub age: u8,
    pub gender: u8,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct OtherClientInfo {
    pub app_id: i64,
    pub instance_id: i32,
    pub sub_platform: String,
    pub device_kind: String,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct RawMessageReceipt {
    pub seqs: Py<PyTuple>,
    pub rands: Py<PyTuple>,
    pub time: i64,
    pub kind: String,
    pub target: i64,
}

#[derive(FromPyObject)]
pub enum OnlineStatusParam {
    #[pyo3(annotation = "tuple[bool, int]")]
    Normal(bool, u64),
    #[pyo3(annotation = "tuple[int, str]")]
    Custom(u64, String),
}

impl From<OnlineStatusParam> for ricq::structs::Status {
    fn from(value: OnlineStatusParam) -> Self {
        use ricq::structs::{CustomOnlineStatus, Status};
        match value {
            OnlineStatusParam::Custom(face_index, wording) => Status {
                online_status: 11,
                ext_online_status: 2000,
                custom_status: Some(CustomOnlineStatus {
                    face_index,
                    wording,
                }),
            },
            OnlineStatusParam::Normal(is_ext, index) => Status {
                online_status: if is_ext { 11 } else { index as i32 },
                ext_online_status: if is_ext { index as i64 } else { 0 },
                custom_status: None,
            },
        }
    }
}
