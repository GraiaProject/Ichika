mod cached;
mod http;
mod params;
pub mod structs;

use std::sync::Arc;
use std::time::Duration;

pub use cached::{cache, ClientCache};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::*;
use ricq::msg::elem::RQElem;
use ricq::structs::{ForwardMessage, FriendAudio, GroupAudio, ProfileDetailUpdate, Status};
use tokio::task::JoinHandle;

use self::http::get_rust_client;
use self::params::*;
use self::structs::*;
use crate::login::{reconnect, TokenRW};
use crate::message::convert::{
    deserialize_message_chain,
    render_forward,
    serialize_audio_dict,
    serialize_element,
    serialize_forward,
};
use crate::message::elements::SealedAudio;
use crate::utils::{py_future, py_none, py_try, py_use, to_py_gender, to_py_permission, AsPython};

#[pyclass(subclass, weakref, module = "ichika.core")]
pub struct PlumbingClient {
    client: Arc<ricq::client::Client>,
    alive: Option<JoinHandle<()>>,
    #[pyo3(get)]
    uin: i64,
    token_rw: TokenRW,
}

/// 用于向 Python 内的 `ichika.client.Client` 传递初始值
#[pyclass]
#[derive(Clone)]
pub struct ClientInitializer {
    pub uin: i64,
    pub client: Arc<ricq::Client>,
    pub alive: Arc<std::sync::Mutex<Option<JoinHandle<()>>>>,
    pub token_rw: TokenRW,
}

#[pymethods]
impl PlumbingClient {
    #[new]
    pub fn new(init: ClientInitializer) -> PyResult<Self> {
        Ok(Self {
            client: init.client,
            alive: init
                .alive
                .lock()
                .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?
                .take(),
            uin: init.uin,
            token_rw: init.token_rw,
        })
    }

    pub fn keep_alive<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let token_rw = self.token_rw.clone();
        let alive = self.alive.take();
        let uin = self.uin;
        py_future(py, async move {
            if let Some(mut alive) = alive {
                loop {
                    alive
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

                    // 断线重连
                    if let Some(handle) = reconnect(&client, &token_rw).await? {
                        alive = handle;
                    } else {
                        break;
                    }
                }
            }
            tracing::info!("客户端 {} 被迫断开连接", uin);
            Ok(py_none())
        })
    }

    #[getter]
    pub fn online(&self) -> bool {
        self.client
            .online
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn stop<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client
                .update_online_status(ricq::structs::OnlineStatus::Offline)
                .await?;
            client.stop(ricq::client::NetworkStatus::Stop);
            Ok(())
        })
    }

    pub fn get_profile<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let profile = client.get_summary_info(uin).await?;
            Ok(Profile::from(profile))
        })
    }

    pub fn get_account_info<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let info = client.account_info.read().await;
            Ok(AccountInfo {
                nickname: info.nickname.clone(),
                age: info.age,
                gender: to_py_gender(info.gender),
            })
        })
    }

    #[pyo3(signature = (*, name=None, email=None, personal_note=None, company=None,college=None,signature=None))]
    #[allow(clippy::too_many_arguments, reason = "Readable")]
    pub fn set_account_info<'py>(
        &self,
        py: Python<'py>,
        name: Option<String>,
        email: Option<String>,
        personal_note: Option<String>,
        company: Option<String>,
        college: Option<String>,
        signature: Option<String>,
    ) -> PyResult<&'py PyAny> {
        macro_rules! set {
            ($field:ident, $profile_update:expr) => {
                if let Some($field) = $field {
                    $profile_update.$field($field);
                }
            };
        }

        let mut upd = ProfileDetailUpdate::new();
        set!(name, upd);
        set!(email, upd);
        set!(personal_note, upd);
        set!(company, upd);
        set!(college, upd);

        let client = self.client.clone();
        py_future(py, async move {
            if !upd.0.is_empty() {
                client.update_profile_detail(upd).await?;
            }
            if let Some(signature) = signature {
                client.update_signature(signature).await?;
            }
            Ok(())
        })
    }

    pub fn get_other_clients<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let other_clients = &*client.online_clients.read().await;
            Python::with_gil(|py| {
                let tup: PyObject = PyTuple::new(
                    py,
                    other_clients
                        .iter()
                        .map(|cl| {
                            OtherClientInfo {
                                app_id: cl.app_id,
                                instance_id: cl.instance_id,
                                sub_platform: cl.sub_platform.clone(),
                                device_kind: cl.device_kind.clone(),
                            }
                            .into_py(py)
                        })
                        .collect::<Vec<PyObject>>(),
                )
                .into_py(py);
                Ok(tup)
            })
        })
    }

    pub fn set_online_status<'py>(
        &self,
        py: Python<'py>,
        status: OnlineStatusParam,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.update_online_status(Status::from(status)).await?;
            Ok(())
        })
    }

    pub fn image_ocr<'py>(
        &self,
        py: Python<'py>,
        url: String,
        md5: String,
        weight: i32,
        height: i32,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let resp = client
                .image_ocr(url, md5, weight * height, weight, height)
                .await?;
            Ok(OCRResult::from(resp).obj())
        })
    }
}

#[pymethods]
impl PlumbingClient {
    #[pyo3(signature = (cache=true))]
    pub fn get_friend_list<'py>(&self, py: Python<'py>, cache: bool) -> PyResult<&'py PyAny> {
        use self::cache as get_cache;

        let use_cache = cache;
        let client = self.client.clone();
        py_future(py, async move {
            let mut cache = get_cache(client).await;
            if !use_cache {
                cache.flush_friend_list().await;
            }
            let friend_list = cache.fetch_friend_list().await?;
            Ok((*friend_list).clone().obj())
        })
    }

    pub fn get_friends<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let friend_list = cache(client).await.fetch_friend_list().await?;
            Ok(Python::with_gil(|py| friend_list.friends(py)))
        })
    }

    pub fn find_friend<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let friend_list = cache(client).await.fetch_friend_list().await?;
            Ok(friend_list.find_friend(uin))
        })
    }

    pub fn nudge_friend<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.friend_poke(uin).await?;
            Ok(())
        })
    }

    pub fn delete_friend<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.delete_friend(uin).await?;
            Ok(())
        })
    }
}

#[pymethods]
impl PlumbingClient {
    #[pyo3(signature = (uin, cache=true))]
    pub fn get_group<'py>(&self, py: Python<'py>, uin: i64, cache: bool) -> PyResult<&'py PyAny> {
        use self::cache as get_cache;
        let use_cache = cache;

        let client = self.client.clone();
        py_future(py, async move {
            let mut cache = get_cache(client).await;
            if !use_cache {
                cache.flush_group(uin).await;
            }
            let group = cache.fetch_group(uin).await?;
            Ok((*group).clone().obj())
        })
    }

    pub fn find_group<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let group = client.get_group_info(uin).await?;
            Ok(group.map(Group::from))
        })
    }

    pub fn get_groups<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let infos = client.get_group_list().await?;
            Ok(py_use(|py| {
                PyTuple::new(
                    py,
                    infos
                        .into_iter()
                        .map(|g| Group::from(g).obj())
                        .collect::<Vec<PyObject>>(),
                )
                .obj()
            }))
        })
    }

    pub fn get_group_admins<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        use self::cache as get_cache;
        let client = self.client.clone();
        py_future(py, async move {
            let admins = client.get_group_admin_list(uin).await?;
            let mut cache = get_cache(client).await;
            let mut admin_list: Vec<Member> = vec![];
            for (member_uin, perm) in admins.iter() {
                let mut member = cache.fetch_member(uin, *member_uin).await?;
                if !member.permission.is(&to_py_permission(perm.clone())) {
                    cache.flush_member(uin, *member_uin).await;
                    member = cache.fetch_member(uin, *member_uin).await?;
                }
                admin_list.push((*member).clone());
            }
            Ok(admin_list)
        })
    }

    pub fn mute_group<'py>(&self, py: Python<'py>, uin: i64, mute: bool) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.group_mute_all(uin, mute).await?;
            Ok(())
        })
    }

    pub fn quit_group<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.group_quit(uin).await?;
            Ok(())
        })
    }

    #[pyo3(signature = (uin, *, memo=None, name=None))]
    pub fn modify_group_info<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        memo: Option<String>,
        name: Option<String>,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            if let Some(memo) = memo {
                client.update_group_memo(uin, memo).await?;
            }
            if let Some(name) = name {
                client.update_group_name(uin, name).await?;
            }
            Ok(())
        })
    }

    pub fn group_sign_in<'py>(&self, py: Python<'py>, uin: i64) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.group_sign_in(uin).await?;
            Ok(())
        })
    }
}

#[pymethods]
impl PlumbingClient {
    #[pyo3(signature = (group_uin, uin, cache=true))]
    pub fn get_member<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        uin: i64,
        cache: bool,
    ) -> PyResult<&'py PyAny> {
        use self::cache as get_cache;
        let use_cache = cache;
        let client = self.client.clone();
        py_future(py, async move {
            let mut cache = get_cache(client).await;
            if !use_cache {
                cache.flush_member(group_uin, uin).await;
            }
            let member = cache.fetch_member(group_uin, uin).await?;
            Ok((*member).clone().obj())
        })
    }

    #[pyo3(signature = (group_uin, cache=true))]
    pub fn get_member_list<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        cache: bool,
    ) -> PyResult<&'py PyAny> {
        use self::cache as get_cache;
        let use_cache = cache;
        let client = self.client.clone();
        py_future(py, async move {
            let mut cache = get_cache(client).await;
            if !use_cache {
                cache.flush_group(group_uin).await;
            }
            let group = cache.fetch_group(group_uin).await?;
            let members = cache
                .client
                .get_group_member_list(group.uin, group.owner_uin)
                .await?;
            let mut guard = cache.detached.lock().await;
            let members = members
                .into_iter()
                .map(|m| {
                    let m = Member::from(m);
                    guard.members.set((group.uin, m.uin), Arc::new(m.clone()));
                    m
                })
                .collect::<Vec<_>>();
            std::mem::drop(guard);
            Ok(members)
        })
    }

    pub fn nudge_member<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        uin: i64,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.group_poke(group_uin, uin).await?;
            Ok(())
        })
    }

    pub fn mute_member<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        uin: i64,
        duration: u64,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client
                .group_mute(group_uin, uin, Duration::from_secs(duration))
                .await?;
            Ok(())
        })
    }

    pub fn kick_member<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        uin: i64,
        msg: String,
        block: bool,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.group_kick(group_uin, vec![uin], &msg, block).await?;
            Ok(())
        })
    }

    pub fn modify_member_special_title<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        uin: i64,
        special_title: String,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client
                .group_edit_special_title(group_uin, uin, special_title)
                .await?;
            Ok(())
        })
    }

    pub fn modify_member_card<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        uin: i64,
        card_name: String,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client
                .edit_group_member_card(group_uin, uin, card_name)
                .await?;
            Ok(())
        })
    }

    pub fn modify_member_admin<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        uin: i64,
        admin: bool,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.group_set_admin(group_uin, uin, admin).await?;
            Ok(())
        })
    }
}

#[pymethods]
impl PlumbingClient {
    pub fn upload_friend_image<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        data: Py<PyBytes>,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let data: Vec<u8> = py_use(|py| data.as_bytes(py).into());
            let image = client.upload_friend_image(uin, &data).await?;
            Ok(py_try(|py| {
                Ok(serialize_element(py, RQElem::FriendImage(image))?.into_py(py))
            })?)
        })
    }

    pub fn upload_friend_audio<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        data: Py<PyBytes>,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let data: Vec<u8> = py_use(|py| data.as_bytes(py).into());
            let audio = client
                .upload_friend_audio(uin, &data, std::time::Duration::from_secs(0))
                .await?;
            let client_uin = client.uin().await;
            let url = client
                .get_friend_audio_url(client_uin, audio.clone())
                .await?;
            Ok(py_try(|py| {
                Ok(serialize_audio_dict(py, url, &audio.0)?.obj())
            })?)
        })
    }

    pub fn upload_group_image<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        data: Py<PyBytes>,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let data: Vec<u8> = py_use(|py| data.as_bytes(py).into());
            let image = client.upload_group_image(uin, &data).await?;
            Ok(py_try(|py| {
                Ok(serialize_element(py, RQElem::GroupImage(image))?.into_py(py))
            })?)
        })
    }

    pub fn upload_group_audio<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        data: Py<PyBytes>,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            let data: Vec<u8> = py_use(|py| data.as_bytes(py).into());
            let audio = client.upload_group_audio(uin, &data, 1).await?;
            let url = client.get_group_audio_url(uin, audio.clone()).await?;
            Ok(py_try(|py| {
                Ok(serialize_audio_dict(py, url, &audio.0)?.obj())
            })?)
        })
    }

    pub fn send_friend_audio<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        audio: PyObject,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let ptt = audio.extract::<SealedAudio>(py)?.inner;
        py_future(py, async move {
            let receipt = client.send_friend_audio(uin, FriendAudio(ptt)).await?;
            Ok(RawMessageReceipt::new(receipt, "friend", uin)?)
        })
    }

    pub fn send_group_audio<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        audio: PyObject,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let ptt = audio.extract::<SealedAudio>(py)?.inner;
        py_future(py, async move {
            let receipt = client.send_group_audio(uin, GroupAudio(ptt)).await?;
            Ok(RawMessageReceipt::new(receipt, "group", uin)?)
        })
    }

    pub fn send_friend_music_share<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        share: MusicShareParam,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let (music_share, music_version) = share.try_into()?;
        py_future(py, async move {
            client
                .send_friend_music_share(uin, music_share, music_version)
                .await?;
            Ok(RawMessageReceipt::empty("group", uin)?)
        })
    }

    pub fn send_group_music_share<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        share: MusicShareParam,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let (music_share, music_version) = share.try_into()?;
        py_future(py, async move {
            client
                .send_group_music_share(uin, music_share, music_version)
                .await?;
            // TODO: Immediate listen hook
            // LINK: https://github.com/Mrs4s/MiraiGo/blob/f8d9841755b579f7c95ed918d23b767e3854553a/client/richmsg.go#L71
            Ok(RawMessageReceipt::empty("group", uin)?)
        })
    }

    pub fn upload_forward_msg<'py>(
        &self,
        py: Python<'py>,
        group_uin: i64,
        msgs: Vec<PyForwardMessage>,
    ) -> PyResult<&'py PyAny> {
        use ricq_core::command::multi_msg::gen_forward_preview;

        let client = self.client.clone();
        let msgs: Vec<ForwardMessage> = msgs.into_iter().map(|v| v.try_into()).try_collect()?;
        let preview = gen_forward_preview(&msgs);
        let summary = format!("查看 {} 条转发消息", msgs.len());
        let file_name: String = "MultiMsg".into();

        py_future(py, async move {
            let res_id = client.upload_msgs(group_uin, msgs, false).await?;
            let content = render_forward(&file_name, &res_id, &preview, &summary);
            Ok((res_id, file_name, content))
        })
    }

    pub fn download_forward_msg<'py>(
        &self,
        py: Python<'py>,
        downloader: &'py PyAny,
        res_id: String,
    ) -> PyResult<&'py PyAny> {
        let mut http_client = get_rust_client(py, downloader)?;
        let client = self.client.clone();

        py_future(py, async move {
            let msgs = client.download_msgs(res_id, &mut http_client).await?;
            Ok(py_try(|py| {
                msgs.into_iter()
                    .map(|msg| serialize_forward(py, msg).map(|ok| ok.into_py(py)))
                    .try_collect::<Vec<PyObject>>()
            })?)
        })
    }
}

#[pymethods]
impl PlumbingClient {
    pub fn send_friend_message<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        chain: &'py PyList,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let chain = deserialize_message_chain(chain)?;
        py_future(py, async move {
            let receipt = client.send_friend_message(uin, chain).await?;
            Ok(RawMessageReceipt::new(receipt, "friend", uin)?)
        })
    }

    pub fn send_group_message<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        chain: &'py PyList,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        let chain = deserialize_message_chain(chain)?;
        py_future(py, async move {
            let receipt = client.send_group_message(uin, chain).await?;
            Ok(RawMessageReceipt::new(receipt, "group", uin)?)
        })
    }

    pub fn recall_friend_message<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        time: i64,
        seq: i32,
        rand: i32,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client
                .recall_friend_message(uin, time, vec![seq], vec![rand])
                .await?;
            Ok(())
        })
    }

    pub fn recall_group_message<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        seq: i32,
        rand: i32,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client
                .recall_group_message(uin, vec![seq], vec![rand])
                .await?;
            Ok(())
        })
    }

    pub fn modify_group_essence<'py>(
        &self,
        py: Python<'py>,
        uin: i64,
        seq: i32,
        rand: i32,
        flag: bool,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client.operate_group_essence(uin, seq, rand, flag).await?;
            Ok(())
        })
    }
}

#[pymethods]
impl PlumbingClient {
    #[allow(clippy::too_many_arguments, reason = "Necessary")]
    pub fn process_join_group_request<'py>(
        &self,
        py: Python<'py>,
        seq: i64,
        request_uin: i64,
        group_uin: i64,
        accept: bool,
        block: bool,
        message: String,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client
                .solve_group_system_message(
                    seq,
                    request_uin,
                    group_uin,
                    false,
                    false,
                    accept,
                    block,
                    message,
                )
                .await?;
            Ok(())
        })
    }

    pub fn process_group_invitation<'py>(
        &self,
        py: Python<'py>,
        seq: i64,
        invitor_uin: i64,
        group_uin: i64,
        accept: bool,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client
                .solve_group_system_message(
                    seq,
                    invitor_uin,
                    group_uin,
                    false,
                    false,
                    accept,
                    false,
                    "".to_string(),
                )
                .await?;
            Ok(())
        })
    }

    pub fn process_new_friend_request<'py>(
        &self,
        py: Python<'py>,
        seq: i64,
        request_uin: i64,
        accept: bool,
    ) -> PyResult<&'py PyAny> {
        let client = self.client.clone();
        py_future(py, async move {
            client
                .solve_friend_system_message(seq, request_uin, accept)
                .await?;
            Ok(())
        })
    }
}

impl Drop for PlumbingClient {
    fn drop(&mut self) {
        let uin = self.uin;
        tracing::info!("{} 的 Rust 客户端已被成功回收", uin);
    }
}
