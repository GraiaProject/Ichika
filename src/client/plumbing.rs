use std::time::Duration;
use std::{path::PathBuf, sync::Arc};

use super::friend::FriendList;
use super::utils::{CacheField, CacheTarget};
use crate::client::structs::*;
use crate::login::reconnect;
use crate::py_intern;
use crate::utils::{py_future, py_none};
use anyhow::Result;
use pyo3::prelude::*;
use tokio::task::JoinHandle;
#[pyclass]
pub struct PlumbingClient {
    client: Arc<ricq::client::Client>,
    alive: Option<JoinHandle<()>>,
    #[pyo3(get)]
    uin: i64,
    data_folder: PathBuf,
    friend_cache: CacheField<FriendList>,
}

impl PlumbingClient {
    #[allow(dead_code)]
    pub async fn new(
        client: Arc<ricq::Client>,
        alive: JoinHandle<()>,
        data_folder: PathBuf,
    ) -> Self {
        let uin = client.uin().await;
        Self {
            client,
            alive: Some(alive),
            uin,
            data_folder,
            friend_cache: CacheField::new(Duration::from_secs(3600)),
        }
    }
}

impl PlumbingClient {
    async fn get_friend_list_raw(&self) -> Result<FriendList> {
        self.friend_cache.clear().await;
        self.friend_cache.get(self.client.clone()).await
    }
    async fn get_friend_list_cached(&self) -> Result<FriendList> {
        self.friend_cache.get(self.client.clone()).await
    }
}

#[pymethods]
impl PlumbingClient {
    pub fn keep_alive<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let data_folder = self.data_folder.clone();
        let alive = self.alive.take();
        let uin = self.uin;
        py_future(py, async move {
            if let Some(mut alive) = alive {
                loop {
                    alive.await?;

                    // 断线重连
                    if let Some(handle) = reconnect(&client, &data_folder).await? {
                        alive = handle;
                    } else {
                        break;
                    }
                }
            }
            tracing::info!("客户端 {} 连接断开", uin);
            Ok(py_none())
        })
    }

    #[getter]
    pub fn online(&self) -> bool {
        self.client
            .online
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn get_account_info<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let info = client.account_info.read().await;
            Ok(AccountInfo {
                nickname: py_intern!(&info.nickname),
                age: info.age,
                gender: info.gender,
            })
        })
    }

    pub fn get_other_clients<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let mut res: Vec<OtherClientInfo> = Vec::new();
            let other_clients = &*client.online_clients.read().await;
            for cl in other_clients.clone() {
                res.push(OtherClientInfo {
                    app_id: cl.app_id,
                    instance_id: cl.instance_id,
                    sub_platform: cl.sub_platform.clone(),
                    device_kind: cl.device_kind.clone(),
                });
            }
            Ok(res)
        })
    }
}
