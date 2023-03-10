use pyo3::prelude::*;
use ricq::client::event as rce;
use ricq::handler::QEvent;

use super::structs::{FriendInfo, MemberInfo, MessageSource};
use super::{FriendMessage, GroupMessage, LoginEvent, TempMessage, UnknownEvent};
use crate::client::cache;
use crate::exc::MapPyErr;
use crate::message::convert::{serialize_as_py_chain, serialize_audio};
use crate::utils::{py_try, py_use};
use crate::PyRet;

pub async fn convert(event: QEvent) -> PyRet {
    match event {
        QEvent::Login(event) => handle_login(event).await,
        QEvent::GroupMessage(event) => handle_group_message(event).await,
        QEvent::GroupAudioMessage(event) => handle_group_audio(event).await,
        QEvent::FriendMessage(event) => handle_friend_message(event).await,
        QEvent::FriendAudioMessage(event) => handle_friend_audio(event).await,
        QEvent::GroupTempMessage(event) => handle_temp_message(event).await,
        unknown => obj(|_| UnknownEvent { inner: unknown }),
    }
}

fn obj<F, R, T>(f: F) -> PyResult<T>
where
    F: for<'py> FnOnce(Python<'py>) -> R,
    R: IntoPy<T>,
{
    py_use(|py| Ok(f(py).into_py(py)))
}

async fn handle_login(uin: i64) -> PyRet {
    obj(|py| LoginEvent { uin }.into_py(py))
}

async fn handle_group_message(event: rce::GroupMessageEvent) -> PyRet {
    let msg = event.inner;

    let mut cache = cache(event.client).await;
    let group_info = cache.fetch_group(msg.group_code).await.py_res()?;
    let sender_info = cache
        .fetch_member(msg.group_code, msg.from_uin)
        .await
        .py_res()?;

    let content = py_try(|py| serialize_as_py_chain(py, msg.elements))?;
    obj(|py| GroupMessage {
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time),
        content,
        sender: MemberInfo {
            uin: msg.from_uin,
            name: sender_info.card_name.clone(),
            nickname: sender_info.nickname.clone(),
            group: (*group_info).clone(),
            permission: sender_info.permission,
        },
    })
}

async fn handle_group_audio(event: rce::GroupAudioMessageEvent) -> PyRet {
    let url = event.url().await.py_res()?;
    let msg = event.inner;
    let content = py_try(|py| serialize_audio(py, url, &msg.audio.0))?;
    let mut cache = cache(event.client).await;
    let group_info = cache.fetch_group(msg.group_code).await.py_res()?;
    let sender_info = cache
        .fetch_member(msg.group_code, msg.from_uin)
        .await
        .py_res()?;

    obj(|py| GroupMessage {
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time),
        content,
        sender: MemberInfo {
            uin: msg.from_uin,
            name: sender_info.card_name.clone(),
            nickname: sender_info.nickname.clone(),
            group: (*group_info).clone(),
            permission: sender_info.permission,
        },
    })
}

async fn handle_friend_message(event: rce::FriendMessageEvent) -> PyRet {
    let msg = event.inner;
    let content = py_try(|py| serialize_as_py_chain(py, msg.elements))?;
    obj(|py| FriendMessage {
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time),
        content,
        sender: FriendInfo {
            uin: msg.from_uin,
            nickname: msg.from_nick,
        },
    })
}

async fn handle_friend_audio(event: rce::FriendAudioMessageEvent) -> PyRet {
    let url = event.url().await.py_res()?;
    let msg = event.inner;
    let content = py_try(|py| serialize_audio(py, url, &msg.audio.0))?;
    obj(|py| FriendMessage {
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time),
        content,
        sender: FriendInfo {
            uin: msg.from_uin,
            nickname: msg.from_nick,
        },
    })
}

async fn handle_temp_message(event: rce::GroupTempMessageEvent) -> PyRet {
    let msg = event.inner;
    let content = py_try(|py| serialize_as_py_chain(py, msg.elements))?;

    let mut cache = cache(event.client).await;
    let group_info = cache.fetch_group(msg.group_code).await.py_res()?;
    let sender_info = cache
        .fetch_member(msg.group_code, msg.from_uin)
        .await
        .py_res()?;

    obj(|py| TempMessage {
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time),
        content,
        sender: MemberInfo {
            uin: msg.from_uin,
            name: sender_info.card_name.clone(),
            nickname: sender_info.nickname.clone(),
            group: (*group_info).clone(),
            permission: sender_info.permission,
        },
    })
}
