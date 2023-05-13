use std::collections::HashMap;

use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_repr::PyRepr;
use ricq::structs::{FriendGroupInfo, FriendInfo, GroupInfo, GroupMemberInfo, MessageReceipt};
use ricq_core::command::friendlist::FriendListResponse;
use ricq_core::command::oidb_svc::OcrResponse;
use ricq_core::structs::SummaryCardInfo;

use crate::utils::{datetime_from_ts, py_try, py_use, to_py_gender, to_py_permission};
#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct AccountInfo {
    pub nickname: String,
    pub age: u8,
    pub gender: PyObject,
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct OtherClientInfo {
    pub app_id: i64,
    pub instance_id: i32,
    pub sub_platform: String,
    pub device_kind: String,
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct RawMessageReceipt {
    pub seq: i32,
    pub rand: i32,
    pub raw_seqs: Py<PyTuple>,
    pub raw_rands: Py<PyTuple>,
    pub time: PyObject, // datetime
    pub kind: String,
    pub target: i64,
}

impl RawMessageReceipt {
    pub fn new(origin: MessageReceipt, kind: impl Into<String>, target: i64) -> PyResult<Self> {
        let kind: String = kind.into();
        let MessageReceipt { seqs, rands, time } = origin;
        let seq: i32 = *seqs
            .first()
            .ok_or_else(|| PyIndexError::new_err("Empty returning seqs"))?;
        let rand: i32 = *rands
            .first()
            .ok_or_else(|| PyIndexError::new_err("Empty returning rands"))?;
        py_try(|py| {
            let time = datetime_from_ts(py, time)?.to_object(py);
            Ok(Self {
                seq,
                rand,
                raw_seqs: PyTuple::new(py, seqs).into_py(py),
                raw_rands: PyTuple::new(py, rands).into_py(py),
                time,
                kind,
                target,
            })
        })
    }

    pub fn empty(kind: impl Into<String>, target: i64) -> PyResult<Self> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_err(|_| PyValueError::new_err("SystemTime before UNIX EPOCH"))?;
        Self::new(
            MessageReceipt {
                seqs: vec![0],
                rands: vec![0],
                time: timestamp.as_secs() as i64,
            },
            kind,
            target,
        )
    }
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct OCRResult {
    pub texts: Py<PyTuple>, // PyTuple<OCRText>
    pub language: String,
}

#[pyclass(get_all, module = "ichika.core")]
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

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct Profile {
    pub uin: i64,
    pub gender: PyObject,
    pub age: u8,
    pub nickname: String,
    pub level: i32,
    pub city: String,
    pub sign: String,
    pub login_days: i64,
}

impl From<SummaryCardInfo> for Profile {
    fn from(value: SummaryCardInfo) -> Self {
        let SummaryCardInfo {
            uin,
            sex,
            age,
            nickname,
            level,
            city,
            sign,
            login_days,
            ..
        } = value;
        Self {
            uin,
            gender: to_py_gender(sex),
            age,
            nickname,
            level,
            city,
            sign,
            login_days,
        }
    }
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct Group {
    pub uin: i64,
    pub name: String,
    pub memo: String,
    pub owner_uin: i64,
    pub create_time: u32,
    pub level: u32,
    pub member_count: u16,
    pub max_member_count: u16,
    // 全群禁言时间
    pub global_mute_timestamp: i64,
    // 自己被禁言时间
    pub mute_timestamp: i64,
    // 最后一条信息的 SEQ,只有通过 GetGroupInfo 函数获取的 GroupInfo 才会有
    pub last_msg_seq: i64,
}

impl From<GroupInfo> for Group {
    fn from(
        GroupInfo {
            code,
            name,
            memo,
            owner_uin,
            group_create_time,
            group_level,
            member_count,
            max_member_count,
            shut_up_timestamp,
            my_shut_up_timestamp,
            last_msg_seq,
            ..
        }: GroupInfo,
    ) -> Self {
        Group {
            uin: code,
            name,
            memo,
            owner_uin,
            create_time: group_create_time,
            level: group_level,
            member_count,
            max_member_count,
            global_mute_timestamp: shut_up_timestamp,
            mute_timestamp: my_shut_up_timestamp,
            last_msg_seq, // TODO: maybe `Option`?
        }
    }
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct Member {
    pub group_uin: i64,
    pub uin: i64,
    pub gender: PyObject,
    pub nickname: String,
    pub raw_card_name: String,
    pub level: u16,
    pub join_time: i64, // TODO: Datetime
    pub last_speak_time: i64,
    pub special_title: String,
    pub special_title_expire_time: i64,
    pub mute_timestamp: i64,
    pub permission: PyObject,
}

impl From<GroupMemberInfo> for Member {
    fn from(
        GroupMemberInfo {
            group_code,
            uin,
            gender,
            nickname,
            card_name,
            level,
            join_time,
            last_speak_time,
            special_title,
            special_title_expire_time,
            shut_up_timestamp,
            permission,
        }: GroupMemberInfo,
    ) -> Self {
        Self {
            group_uin: group_code,
            uin,
            gender: to_py_gender(gender),
            nickname,
            raw_card_name: card_name,
            level,
            join_time,
            last_speak_time,
            special_title,
            special_title_expire_time,
            mute_timestamp: shut_up_timestamp,
            permission: to_py_permission(permission),
        }
    }
}

#[pymethods]
impl Member {
    #[getter]
    fn card_name(&self) -> String {
        if self.raw_card_name.is_empty() {
            self.nickname.clone()
        } else {
            self.raw_card_name.clone()
        }
    }
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct Friend {
    pub uin: i64,
    pub nick: String,
    pub remark: String,
    pub face_id: i16,
    pub group_id: u8,
}

impl From<FriendInfo> for Friend {
    fn from(info: FriendInfo) -> Self {
        Friend {
            uin: info.uin,
            nick: info.nick,
            remark: info.remark,
            face_id: info.face_id,
            group_id: info.group_id,
        }
    }
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct FriendGroup {
    pub group_id: u8,
    pub name: String,
    pub total_count: i32,
    pub online_count: i32,
    pub seq_id: u8,
}

impl From<FriendGroupInfo> for FriendGroup {
    fn from(
        FriendGroupInfo {
            group_id,
            group_name,
            friend_count,
            online_friend_count,
            seq_id,
        }: FriendGroupInfo,
    ) -> Self {
        FriendGroup {
            group_id,
            name: group_name,
            total_count: friend_count,
            online_count: online_friend_count,
            seq_id,
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct FriendList {
    entries: Vec<Friend>,
    friend_groups: HashMap<u8, FriendGroup>,
    #[pyo3(get)]
    pub total_count: i16,
    #[pyo3(get)]
    pub online_count: i16,
}

#[pymethods]
impl FriendList {
    pub fn friends(&self, py: Python) -> Py<PyTuple> {
        PyTuple::new(
            py,
            self.entries
                .clone()
                .into_iter()
                .map(|f| f.into_py(py))
                .collect::<Vec<PyObject>>(),
        )
        .into_py(py)
    }

    pub fn find_friend(&self, uin: i64) -> Option<Friend> {
        self.entries
            .iter()
            .find(|friend| friend.uin == uin)
            .cloned()
    }

    pub fn friend_groups(&self, py: Python) -> Py<PyTuple> {
        PyTuple::new(
            py,
            self.friend_groups
                .clone()
                .into_values()
                .map(|g| g.into_py(py))
                .collect::<Vec<PyObject>>(),
        )
        .into_py(py)
    }

    pub fn find_friend_group(&self, group_id: u8) -> Option<FriendGroup> {
        self.friend_groups.get(&group_id).cloned()
    }
}

impl From<FriendListResponse> for FriendList {
    fn from(resp: FriendListResponse) -> Self {
        Self {
            entries: resp.friends.into_iter().map(Friend::from).collect(),
            friend_groups: resp
                .friend_groups
                .into_iter()
                .map(|(g_id, info)| (g_id, FriendGroup::from(info)))
                .collect(),
            total_count: resp.total_count,
            online_count: resp.online_friend_count,
        }
    }
}
