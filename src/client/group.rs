use pyo3::prelude::*;
use ricq::structs::GroupInfo;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Group {
    #[pyo3(get)]
    pub uin: i64,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub memo: String,
    #[pyo3(get)]
    pub owner_uin: i64,
    #[pyo3(get)]
    pub create_time: u32,
    #[pyo3(get)]
    pub level: u32,
    #[pyo3(get)]
    pub member_count: u16,
    #[pyo3(get)]
    pub max_member_count: u16,
    // 全群禁言时间
    #[pyo3(get)]
    pub global_mute_timestamp: i64,
    // 自己被禁言时间
    #[pyo3(get)]
    pub mute_timestamp: i64,
    // 最后一条信息的 SEQ,只有通过 GetGroupInfo 函数获取的 GroupInfo 才会有
    #[pyo3(get)]
    pub last_msg_seq: i64,
}

crate::repr!(Group);

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
