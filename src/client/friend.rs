use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::*;
use pyo3_repr::PyRepr;
use ricq::structs::{FriendGroupInfo, FriendInfo};
use ricq_core::command::friendlist::FriendListResponse;

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct Friend {
    uin: i64,
    nick: String,
    remark: String,
    face_id: i16,
    group_id: u8,
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

#[pyclass(get_all)]
#[derive(PyRepr, Clone)]
pub struct FriendGroup {
    group_id: u8,
    name: String,
    total_count: i32,
    online_count: i32,
    seq_id: u8,
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
    total_count: i16,
    #[pyo3(get)]
    online_count: i16,
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
