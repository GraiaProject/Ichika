use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use ricq::client::event as rce;
use ricq::handler::QEvent;

use super::structs::{FriendInfo, MessageSource};
use super::{
    FriendDeleted,
    FriendMessage,
    FriendNudge,
    FriendRecallMessage,
    GroupDisband,
    GroupInfoUpdate,
    GroupMessage,
    GroupMute,
    GroupNudge,
    GroupRecallMessage,
    LoginEvent,
    MemberLeaveGroup,
    MemberMute,
    MemberPermissionChange,
    NewFriend,
    NewMember,
    TempMessage,
    UnknownEvent,
};
use crate::client::cache;
use crate::exc::MapPyErr;
use crate::message::convert::{serialize_as_py_chain, serialize_audio};
use crate::utils::{datetime_from_ts, py_none, py_try, py_use, timedelta_from_secs, AsPython};
use crate::{py_dict, PyRet};

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
        QEvent::NewFriend(event) => Ok(handle_new_friend(event)),
        QEvent::NewMember(event) => handle_new_member(event).await,
        QEvent::GroupLeave(event) => Ok(handle_group_leave(event).await),
        QEvent::GroupDisband(event) => Ok(handle_group_disband(event).await),
        QEvent::DeleteFriend(event) => Ok(handle_friend_delete(event).await),
        QEvent::GroupMute(event) => handle_mute(event).await,
        QEvent::MemberPermissionChange(event) => handle_permission_change(event).await,
        QEvent::GroupNameUpdate(event) => handle_group_info_update(event).await,
        unknown => Ok(UnknownEvent { inner: unknown }.obj()),
    }
}

fn handle_login(uin: i64) -> PyObject {
    LoginEvent { uin }.obj()
}

// TODO: split `fetch_group` and `fetch_member` into helper functions

async fn handle_group_message(event: rce::GroupMessageEvent) -> PyRet {
    let msg = event.inner;

    let mut cache = cache(event.client).await;
    let group = cache
        .fetch_group(msg.group_code)
        .await
        .py_res()?
        .as_ref()
        .clone();
    let sender = cache
        .fetch_member(msg.group_code, msg.from_uin)
        .await
        .py_res()?
        .as_ref()
        .clone();

    let content = py_try(|py| serialize_as_py_chain(py, msg.elements))?;
    py_try(|py| {
        Ok(GroupMessage {
            source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
            content,
            group,
            sender,
        }
        .obj())
    })
}

async fn handle_group_recall(event: rce::GroupMessageRecallEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    let group = cache
        .fetch_group(event.group_code)
        .await
        .py_res()?
        .as_ref()
        .clone();
    let author = cache
        .fetch_member(event.group_code, event.author_uin)
        .await
        .py_res()?
        .as_ref()
        .clone();
    let operator = cache
        .fetch_member(event.group_code, event.operator_uin)
        .await
        .py_res()?
        .as_ref()
        .clone();
    let time = py_try(|py| Ok(datetime_from_ts(py, event.time)?.into_py(py)))?;
    Ok(GroupRecallMessage {
        time,
        group,
        author,
        operator,
        seq: event.msg_seq,
    }
    .obj())
}

async fn handle_group_audio(event: rce::GroupAudioMessageEvent) -> PyRet {
    let url = event.url().await.py_res()?;
    let msg = event.inner;
    let content = py_try(|py| serialize_audio(py, url, &msg.audio.0))?;
    let mut cache = cache(event.client).await;
    let group = cache
        .fetch_group(msg.group_code)
        .await
        .py_res()?
        .as_ref()
        .clone();
    let sender = cache
        .fetch_member(msg.group_code, msg.from_uin)
        .await
        .py_res()?
        .as_ref()
        .clone();

    py_try(|py| {
        Ok(GroupMessage {
            source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
            content,
            group,
            sender,
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
    let time = py_try(|py| Ok(datetime_from_ts(py, event.time)?.into_py(py)))?;
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
    let group = cache
        .fetch_group(msg.group_code)
        .await
        .py_res()?
        .as_ref()
        .clone();
    let sender = cache
        .fetch_member(msg.group_code, msg.from_uin)
        .await
        .py_res()?
        .as_ref()
        .clone();

    py_try(|py| {
        Ok(TempMessage {
            source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
            content,
            group,
            sender,
        }
        .obj())
    })
}

async fn handle_group_nudge(event: rce::GroupPokeEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    let group = cache
        .fetch_group(event.group_code)
        .await
        .py_res()?
        .as_ref()
        .clone();
    let sender = cache
        .fetch_member(event.group_code, event.sender)
        .await
        .py_res()?
        .as_ref()
        .clone();
    let receiver = cache
        .fetch_member(event.group_code, event.receiver)
        .await
        .py_res()?
        .as_ref()
        .clone();

    Ok(GroupNudge {
        group,
        sender,
        receiver,
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

fn handle_new_friend(event: rce::NewFriendEvent) -> PyObject {
    NewFriend {
        friend: FriendInfo {
            uin: event.inner.uin,
            nickname: event.inner.nick,
        },
    }
    .obj()
}
async fn handle_new_member(event: rce::NewMemberEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    let group = cache
        .fetch_group(event.group_code)
        .await
        .py_res()?
        .as_ref()
        .clone();
    let member = cache
        .fetch_member(event.group_code, event.member_uin)
        .await
        .py_res()?
        .as_ref()
        .clone();
    Ok(NewMember { group, member }.obj())
}

async fn handle_group_leave(event: rce::GroupLeaveEvent) -> PyObject {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    cache.flush_member(event.group_code, event.member_uin).await;

    MemberLeaveGroup {
        group_uin: event.group_code,
        member_uin: event.member_uin,
    }
    .obj()
}

async fn handle_group_disband(event: rce::GroupDisbandEvent) -> PyObject {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    cache.flush_group(event.group_code).await;
    GroupDisband {
        group_uin: event.group_code,
        operator_uin: event.operator_uin,
    }
    .obj()
}

async fn handle_friend_delete(event: rce::DeleteFriendEvent) -> PyObject {
    let mut cache = cache(event.client).await;
    cache.flush_friend_list().await;
    FriendDeleted {
        friend_uin: event.inner.uin,
    }
    .obj()
}

async fn handle_mute(event: rce::GroupMuteEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    cache.flush_group(event.group_code).await;
    let group = cache
        .fetch_group(event.group_code)
        .await
        .py_res()?
        .as_ref()
        .clone();

    let operator = cache
        .fetch_member(event.group_code, event.operator_uin)
        .await
        .py_res()?
        .as_ref()
        .clone();

    if event.target_uin == 0 {
        return Ok(GroupMute {
            group,
            operator,
            status: event.duration.as_secs() == 0,
        }
        .obj());
    }
    let duration = event.duration.as_secs();
    let duration = py_try(|py| {
        Ok(if duration != 0 {
            timedelta_from_secs(py, duration)?.into_py(py)
        } else {
            false.into_py(py)
        })
    })?;
    let target = cache
        .fetch_member(event.group_code, event.target_uin)
        .await
        .py_res()?
        .as_ref()
        .clone();
    Ok(MemberMute {
        group,
        operator,
        target,
        duration,
    }
    .obj())
}

async fn handle_permission_change(event: rce::MemberPermissionChangeEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    cache.flush_member(event.group_code, event.member_uin).await;
    let group = cache
        .fetch_group(event.group_code)
        .await
        .py_res()?
        .as_ref()
        .clone();
    let target = cache
        .fetch_member(event.group_code, event.member_uin)
        .await
        .py_res()?
        .as_ref()
        .clone();
    Ok(MemberPermissionChange {
        group,
        target,
        permission: event.new_permission as u8,
    }
    .obj())
}

async fn handle_group_info_update(event: rce::GroupNameUpdateEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    cache.flush_group(event.group_code).await;
    let group = cache
        .fetch_group(event.group_code)
        .await
        .py_res()?
        .as_ref()
        .clone();
    let operator = cache
        .fetch_member(event.group_code, event.operator_uin)
        .await
        .py_res()?
        .as_ref()
        .clone();
    Ok(GroupInfoUpdate {
        group,
        operator,
        info: py_use(|py| py_dict!(py, "name" => event.group_name).into_py(py)),
    }
    .obj())
}
