use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_repr::PyRepr;
use ricq::structs::{MusicShare, MusicVersion};
use ricq_core::command::oidb_svc::OcrResponse;

use crate::utils::py_use;
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

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct OCRResult {
    pub texts: Py<PyTuple>, // PyTuple<OCRText>
    pub language: String,
}

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct OCRText {
    pub detected_text: String,
    pub confidence: i32,
    pub polygon: Option<Py<PyTuple>>, // PyTuple<(i32, i32))>
    pub advanced_info: String,
}

impl From<OcrResponse> for OCRResult {
    fn from(value: OcrResponse) -> Self {
        py_use(|py| {
            let OcrResponse { texts, language } = value;
            let text_iter = texts.into_iter().map(|txt| {
                let polygon = txt.polygon.map(|poly| {
                    PyTuple::new(
                        py,
                        poly.coordinates
                            .into_iter()
                            .map(|coord| (coord.x, coord.y).to_object(py)),
                    )
                    .into_py(py)
                });
                OCRText {
                    detected_text: txt.detected_text,
                    confidence: txt.confidence,
                    polygon,
                    advanced_info: txt.advanced_info,
                }
                .into_py(py)
            });
            OCRResult {
                texts: PyTuple::new(py, text_iter).into_py(py),
                language,
            }
        })
    }
}

#[derive(FromPyObject)]
pub enum OnlineStatusParam {
    #[pyo3(annotation = "tuple[bool, int]")]
    Normal(bool, i32),
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
                online_status: if is_ext { 11 } else { index },
                ext_online_status: if is_ext { i64::from(index) } else { 0 },
                custom_status: None,
            },
        }
    }
}

#[derive(FromPyObject)]
pub struct MusicShareParam {
    #[pyo3(attribute)]
    kind: String,
    #[pyo3(attribute)]
    title: String,
    #[pyo3(attribute)]
    summary: String,
    #[pyo3(attribute)]
    jump_url: String,
    #[pyo3(attribute)]
    picture_url: String,
    #[pyo3(attribute)]
    music_url: String,
    #[pyo3(attribute)]
    brief: String,
}

impl TryFrom<MusicShareParam> for (MusicShare, MusicVersion) {
    type Error = PyErr;

    fn try_from(value: MusicShareParam) -> Result<Self, Self::Error> {
        let MusicShareParam {
            kind,
            title,
            summary,
            jump_url,
            picture_url,
            music_url,
            brief,
        } = value;
        let version = match kind.as_str() {
            "QQ" => MusicVersion::QQ,
            "Netease" => MusicVersion::NETEASE,
            "Migu" => MusicVersion::MIGU,
            "Kugou" => MusicVersion::KUGOU,
            "Kuwo" => MusicVersion::KUWO,
            platform => {
                return Err(PyValueError::new_err(format!(
                    "无法识别的音乐平台: {platform}"
                )))
            }
        };
        let share = MusicShare {
            title,
            brief,
            summary,
            url: jump_url,
            picture_url,
            music_url,
        };
        Ok((share, version))
    }
}
