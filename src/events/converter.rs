use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use ricq::client::event as rce;
use ricq::handler::QEvent;

use super::structs::{FriendInfo, MemberInfo, MessageSource};
use super::{
    FriendMessage,
    FriendNudge,
    FriendRecallMessage,
    GroupMessage,
    GroupNudge,
    GroupRecallMessage,
    LoginEvent,
    TempMessage,
    UnknownEvent,
};
use crate::client::cache;
use crate::exc::MapPyErr;
use crate::message::convert::{serialize_as_py_chain, serialize_audio};
use crate::utils::{datetime_from_ts, py_none, py_try, AsPython};
use crate::{call_static_py, PyRet};

pub async fn convert(event: QEvent) -> PyRet {
    match event {
        QEvent::Login(event) => Ok(handle_login(event)),
        QEvent::GroupMessage(event) => handle_group_message(event).await,
        QEvent::GroupAudioMessage(event) => handle_group_audio(event).await,
        QEvent::FriendMessage(event) => handle_friend_message(event),
        QEvent::FriendAudioMessage(event) => handle_friend_audio(event).await,
        QEvent::GroupTempMessage(event) => handle_temp_message(event).await,
        QEvent::GroupMessageRecall(event) => handle_group_recall(event).await,
        QEvent::FriendMessageRecall(event) => handle_friend_recall(event).await,
        QEvent::GroupPoke(event) => handle_group_nudge(event).await,
        QEvent::FriendPoke(event) => handle_friend_nudge(event).await,
        unknown => Ok(UnknownEvent { inner: unknown }.obj()),
    }
}

fn handle_login(uin: i64) -> PyObject {
    LoginEvent { uin }.obj()
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
    py_try(|py| {
        Ok(GroupMessage {
            source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
            content,
            sender: MemberInfo {
                uin: msg.from_uin,
                name: sender_info.card_name.clone(),
                nickname: sender_info.nickname.clone(),
                group: (*group_info).clone(),
                permission: sender_info.permission,
            },
        }
        .obj())
    })
}

async fn handle_group_recall(event: rce::GroupMessageRecallEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    let group_info = cache.fetch_group(event.group_code).await.py_res()?;
    let author = cache
        .fetch_member(event.group_code, event.author_uin)
        .await
        .py_res()?;
    let operator = cache
        .fetch_member(event.group_code, event.operator_uin)
        .await
        .py_res()?;
    let time = py_try(|py| Ok(call_static_py!(datetime_from_ts, py, (event.time))?.into_py(py)))?;
    Ok(GroupRecallMessage {
        time,
        author: MemberInfo::new(&author, (*group_info).clone()),
        operator: MemberInfo::new(&operator, (*group_info).clone()),
        seq: event.msg_seq,
    }
    .obj())
}

async fn handle_group_audio(event: rce::GroupAudioMessageEvent) -> PyRet {
    let url = event.url().await.py_res()?;
    let msg = event.inner;
    let content = py_try(|py| serialize_audio(py, url, &msg.audio.0))?;
    let mut cache = cache(event.client).await;
    let group = cache.fetch_group(msg.group_code).await.py_res()?;
    let sender = cache
        .fetch_member(msg.group_code, msg.from_uin)
        .await
        .py_res()?;

    py_try(|py| {
        Ok(GroupMessage {
            source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
            content,
            sender: MemberInfo::new(&sender, (*group).clone()),
        }
        .obj())
    })
}

fn handle_friend_message(event: rce::FriendMessageEvent) -> PyRet {
    let msg = event.inner;
    let content = py_try(|py| serialize_as_py_chain(py, msg.elements))?;
    py_try(|py| {
        Ok(FriendMessage {
            source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
            content,
            sender: FriendInfo {
                uin: msg.from_uin,
                nickname: msg.from_nick,
            },
        }
        .obj())
    })
}

async fn handle_friend_recall(event: rce::FriendMessageRecallEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    let friend = cache
        .fetch_friend_list()
        .await
        .py_res()?
        .find_friend(event.friend_uin)
        .ok_or_else(|| {
            PyValueError::new_err(format!("Unable to find friend {}", event.friend_uin))
        })?;
    let time = py_try(|py| Ok(call_static_py!(datetime_from_ts, py, (event.time))?.into_py(py)))?;
    Ok(FriendRecallMessage {
        time,
        author: FriendInfo {
            uin: friend.uin,
            nickname: friend.nick,
        },
        seq: event.msg_seq,
    }
    .obj())
}

async fn handle_friend_audio(event: rce::FriendAudioMessageEvent) -> PyRet {
    let url = event.url().await.py_res()?;
    let msg = event.inner;
    let content = py_try(|py| serialize_audio(py, url, &msg.audio.0))?;
    py_try(|py| {
        Ok(FriendMessage {
            source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
            content,
            sender: FriendInfo {
                uin: msg.from_uin,
                nickname: msg.from_nick,
            },
        }
        .obj())
    })
}

async fn handle_temp_message(event: rce::GroupTempMessageEvent) -> PyRet {
    let msg = event.inner;
    let content = py_try(|py| serialize_as_py_chain(py, msg.elements))?;

    let mut cache = cache(event.client).await;
    let group = cache.fetch_group(msg.group_code).await.py_res()?;
    let sender = cache
        .fetch_member(msg.group_code, msg.from_uin)
        .await
        .py_res()?;

    py_try(|py| {
        Ok(TempMessage {
            source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
            content,
            sender: MemberInfo::new(&sender, (*group).clone()),
        }
        .obj())
    })
}

async fn handle_group_nudge(event: rce::GroupPokeEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    let group = cache.fetch_group(event.group_code).await.py_res()?;
    let sender = cache
        .fetch_member(event.group_code, event.sender)
        .await
        .py_res()?;
    let receiver = cache
        .fetch_member(event.group_code, event.receiver)
        .await
        .py_res()?;

    Ok(GroupNudge {
        sender: MemberInfo::new(&sender, (*group).clone()),
        receiver: MemberInfo::new(&receiver, (*group).clone()),
    }
    .obj())
}

async fn handle_friend_nudge(event: rce::FriendPokeEvent) -> PyRet {
    let client = event.client;
    if client.uin().await == event.inner.sender {
        return Ok(py_none());
    }
    let mut cache = cache(client).await;
    let event = event.inner;
    let friend = cache
        .fetch_friend_list()
        .await
        .py_res()?
        .find_friend(event.sender)
        .ok_or_else(|| PyValueError::new_err(format!("Unable to find friend {}", event.sender)))?;
    Ok(FriendNudge {
        sender: FriendInfo {
            uin: friend.uin,
            nickname: friend.nick,
        },
    }
    .obj())
}
