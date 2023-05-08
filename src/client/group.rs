use pyo3::prelude::*;
use pyo3_repr::PyRepr;
use ricq::structs::{GroupInfo, GroupMemberInfo};

use crate::utils::{datetime_from_ts, py_try};
#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct Group {
    pub uin: i64,
    pub name: String,
    pub memo: String,
    pub owner_uin: i64,
    pub create_time: PyObject,
    pub level: u32,
    pub member_count: u16,
    pub max_member_count: u16,
}

impl TryFrom<GroupInfo> for Group {
    type Error = PyErr;

    fn try_from(
        GroupInfo {
            code,
            name,
            memo,
            owner_uin,
            group_create_time,
            group_level,
            member_count,
            max_member_count,
            ..
        }: GroupInfo,
    ) -> PyResult<Self> {
        Ok(Self {
            uin: code,
            name,
            memo,
            owner_uin,
            create_time: py_try(|py| Ok(datetime_from_ts(py, group_create_time)?.to_object(py)))?,
            level: group_level,
            member_count,
            max_member_count,
        })
    }
}

#[pyclass(get_all, module = "ichika.core")]
#[derive(PyRepr, Clone)]
pub struct Member {
    pub group_uin: i64,
    pub uin: i64,
    pub gender: u8,
    pub nickname: String,
    pub raw_card_name: String,
    pub level: u16,
    pub join_time: PyObject,
    pub last_speak_time: PyObject,
    pub special_title: String,
    pub special_title_expire_time: PyObject,
    pub mute_timestamp: PyObject,
    pub permission: u8,
}

impl TryFrom<GroupMemberInfo> for Member {
    type Error = PyErr;

    fn try_from(
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
    ) -> PyResult<Self> {
        Ok(Self {
            group_uin: group_code,
            uin,
            gender,
            nickname,
            raw_card_name: card_name,
            level,
            join_time: py_try(|py| Ok(datetime_from_ts(py, join_time)?.to_object(py)))?,
            last_speak_time: py_try(|py| Ok(datetime_from_ts(py, last_speak_time)?.to_object(py)))?,
            special_title,
            special_title_expire_time: py_try(|py| {
                Ok(datetime_from_ts(py, special_title_expire_time)?.to_object(py))
            })?,
            mute_timestamp: py_try(
                |py| Ok(datetime_from_ts(py, shut_up_timestamp)?.to_object(py)),
            )?,
            permission: permission as u8,
        })
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
