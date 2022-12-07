use anyhow::Result;
use futures_util::Future;
use pyo3::prelude::*;
use ricq::{
    structs::{FriendGroupInfo, FriendInfo},
    Client,
};
use std::{collections::HashMap, sync::Arc};

use super::utils::CacheTarget;
#[pyclass(module = "ichika.client.structs.friend#rs")]
#[derive(Debug, Clone)]
pub struct Friend {
    #[pyo3(get)]
    uin: i64,
    #[pyo3(get)]
    nick: String,
    #[pyo3(get)]
    remark: String,
    #[pyo3(get)]
    face_id: i16,
    #[pyo3(get)]
    group_id: u8,
}

crate::repr!(Friend);

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

#[pyclass(module = "ichika.client.structs.friend#rs")]
#[derive(Debug, Clone)]
pub struct FriendGroup {
    #[pyo3(get)]
    group_id: u8,
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    total_count: i32,
    #[pyo3(get)]
    online_count: i32,
    #[pyo3(get)]
    seq_id: u8,
}

crate::repr!(FriendGroup);

impl From<FriendGroupInfo> for FriendGroup {
    fn from(info: FriendGroupInfo) -> Self {
        FriendGroup {
            group_id: info.group_id,
            name: info.group_name,
            total_count: info.friend_count,
            online_count: info.online_friend_count,
            seq_id: info.seq_id,
        }
    }
}

#[pyclass(module = "ichika.client.structs.friend#rs")]
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
    pub fn friends(&self, py: Python) -> Py<pyo3::types::PyTuple> {
        pyo3::types::PyTuple::new(
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

    pub fn friend_groups(&self, py: Python) -> Py<pyo3::types::PyTuple> {
        pyo3::types::PyTuple::new(
            py,
            self.friend_groups
                .clone()
                .into_iter()
                .map(|(_, g)| g.into_py(py))
                .collect::<Vec<PyObject>>(),
        )
        .into_py(py)
    }

    pub fn find_friend_group(&self, group_id: u8) -> Option<FriendGroup> {
        self.friend_groups.get(&group_id).cloned()
    }
}

impl CacheTarget for FriendList {
    type FetchFuture = impl Future<Output = Result<Self>>;

    fn fetch(client: Arc<Client>) -> Self::FetchFuture {
        async move {
            let resp = client.get_friend_list().await?;
            let friend_list = FriendList {
                entries: Vec::from_iter(resp.friends.into_iter().map(|f| Friend::from(f))),
                friend_groups: HashMap::from_iter(
                    resp.friend_groups
                        .into_iter()
                        .map(|(g_id, info)| (g_id, FriendGroup::from(info))),
                ),
                total_count: resp.total_count,
                online_count: resp.online_friend_count,
            };
            Ok(friend_list)
        }
    }
}
