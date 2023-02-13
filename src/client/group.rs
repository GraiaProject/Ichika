use pyo3::prelude::*;
use pyo3_repr::PyRepr;
use ricq::structs::GroupInfo;
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
    fn from(info: GroupInfo) -> Self {
        Group {
            uin: info.code,
            name: info.name,
            memo: info.memo,
            owner_uin: info.owner_uin,
            create_time: info.group_create_time,
            level: info.group_level,
            member_count: info.member_count,
            max_member_count: info.max_member_count,
            global_mute_timestamp: info.shut_up_timestamp,
            mute_timestamp: info.my_shut_up_timestamp,
            last_msg_seq: info.last_msg_seq, // TODO: maybe `Option`?
        }
    }
}
