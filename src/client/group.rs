use pyo3::prelude::*;
use pyo3_repr::PyRepr;
use ricq::structs::{GroupInfo, GroupMemberInfo};
#[pyclass(get_all)]
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

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct Member {
    pub group_uin: i64,
    pub uin: i64,
    pub gender: u8,
    pub nickname: String,
    pub card_name: String,
    pub level: u16,
    pub join_time: i64, // TODO: Datetime
    pub last_speak_time: i64,
    pub special_title: String,
    pub special_title_expire_time: i64,
    pub mute_timestamp: i64,
    pub permission: u8,
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
            gender,
            nickname,
            card_name,
            level,
            join_time,
            last_speak_time,
            special_title,
            special_title_expire_time,
            mute_timestamp: shut_up_timestamp,
            permission: permission as u8,
        }
    }
}
