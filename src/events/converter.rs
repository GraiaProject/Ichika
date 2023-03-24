use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use ricq::client::event as rce;
use ricq::handler::QEvent;

use super::structs::{FriendInfo, MessageSource};
use crate::client::cache;
use crate::exc::MapPyErr;
use crate::message::convert::{serialize_as_py_chain, serialize_audio};
use crate::utils::{datetime_from_ts, py_none, py_try, timedelta_from_secs};
use crate::{dict_obj, PyRet};

pub async fn convert(event: QEvent) -> PyRet {
    match event {
        QEvent::Login(event) => handle_login(event),
        QEvent::GroupMessage(event) => handle_group_message(event).await,
        QEvent::GroupAudioMessage(event) => handle_group_audio(event).await,
        QEvent::FriendMessage(event) => handle_friend_message(event),
        QEvent::FriendAudioMessage(event) => handle_friend_audio(event).await,
        QEvent::GroupTempMessage(event) => handle_temp_message(event).await,
        QEvent::GroupMessageRecall(event) => handle_group_recall(event).await,
        QEvent::FriendMessageRecall(event) => handle_friend_recall(event).await,
        QEvent::GroupPoke(event) => handle_group_nudge(event).await,
        QEvent::FriendPoke(event) => handle_friend_nudge(event).await,
        QEvent::NewFriend(event) => handle_new_friend(event),
        QEvent::NewMember(event) => handle_new_member(event).await,
        QEvent::GroupLeave(event) => handle_group_leave(event).await,
        QEvent::GroupDisband(event) => handle_group_disband(event).await,
        QEvent::DeleteFriend(event) => handle_friend_delete(event).await,
        QEvent::GroupMute(event) => handle_mute(event).await,
        QEvent::MemberPermissionChange(event) => handle_permission_change(event).await,
        QEvent::GroupNameUpdate(event) => handle_group_info_update(event).await,
        QEvent::GroupRequest(event) => handle_group_request(event),
        QEvent::SelfInvited(event) => handle_group_invitation(event),
        QEvent::NewFriendRequest(event) => handle_friend_request(event),
        unknown => dict_obj!(type_name: "UnknownEvent", internal_repr: format!("{:?}", unknown)),
    }
}

fn handle_login(uin: i64) -> PyRet {
    dict_obj! {
        type_name: "LoginAttempt",
        uin: uin
    }
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
    dict_obj! {py !
        type_name: "GroupMessage",
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
        content: content,
        group: group,
        sender: sender,
    }
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
    dict_obj! {
        type_name: "GroupRecallMessage",
        time: time,
        group: group,
        author: author,
        operator: operator,
        seq: event.msg_seq,
    }
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

    dict_obj! {py !
        type_name: "GroupMessage",
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
        content: content,
        group: group,
        sender: sender,
    }
}

fn handle_friend_message(event: rce::FriendMessageEvent) -> PyRet {
    let msg = event.inner;
    let content = py_try(|py| serialize_as_py_chain(py, msg.elements))?;
    dict_obj! {py !
        type_name: "FriendMessage",
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
        content: content,
        sender: FriendInfo {
            uin: msg.from_uin,
            nickname: msg.from_nick,
        },
    }
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
    dict_obj! {
        type_name: "FriendRecallMessage",
        time: time,
        author: FriendInfo {
            uin: friend.uin,
            nickname: friend.nick,
        },
        seq: event.msg_seq,
    }
}

async fn handle_friend_audio(event: rce::FriendAudioMessageEvent) -> PyRet {
    let url = event.url().await.py_res()?;
    let msg = event.inner;
    let content = py_try(|py| serialize_audio(py, url, &msg.audio.0))?;
    dict_obj! {py !
        type_name: "FriendMessage",
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
        content: content,
        sender: FriendInfo {
            uin: msg.from_uin,
            nickname: msg.from_nick,
        },
    }
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

    dict_obj! {py !
        type_name: "TempMessage",
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
        content: content,
        group: group,
        sender: sender,
    }
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

    dict_obj! {
        type_name: "GroupNudge",
        group: group,
        sender: sender,
        receiver: receiver,
    }
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
    dict_obj! {
        type_name: "FriendNudge",
        sender: FriendInfo {
            uin: friend.uin,
            nickname: friend.nick,
        },
    }
}

fn handle_new_friend(event: rce::NewFriendEvent) -> PyRet {
    dict_obj! {
        type_name: "NewFriend",
        friend: FriendInfo {
            uin: event.inner.uin,
            nickname: event.inner.nick,
        },
    }
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
    dict_obj! {
        type_name: "NewMember",
        group: group,
        member: member,
    }
}

async fn handle_group_leave(event: rce::GroupLeaveEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    cache.flush_member(event.group_code, event.member_uin).await;

    dict_obj! {
        type_name: "MemberLeaveGroup",
        group_uin: event.group_code,
        member_uin: event.member_uin,
    }
}

async fn handle_group_disband(event: rce::GroupDisbandEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    let event = event.inner;
    cache.flush_group(event.group_code).await;
    dict_obj! {
        type_name: "GroupDisband",
        group_uin: event.group_code,
        operator_uin: event.operator_uin,
    }
}

async fn handle_friend_delete(event: rce::DeleteFriendEvent) -> PyRet {
    let mut cache = cache(event.client).await;
    cache.flush_friend_list().await;
    dict_obj! {
        type_name: "FriendDeleted",
        friend_uin: event.inner.uin,
    }
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
        return dict_obj! {
            type_name: "MemberMute",
            group: group,
            operator: operator,
            duration: event.duration.as_secs() == 0
        };
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
    dict_obj! {
        type_name: "MemberMute",
        group: group,
        operator: operator,
        target: target,
        duration: duration,
    }
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
    dict_obj! {
        type_name: "MemberPermissionChange",
        group: group,
        target: target,
        permission: event.new_permission as u8,
    }
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
    dict_obj! {
        type_name: "GroupInfoUpdate",
        group: group,
        operator: operator,
        info: dict_obj! {
            name: event.group_name
        }?,
    }
}

fn handle_group_request(event: rce::JoinGroupRequestEvent) -> PyRet {
    let event = event.inner;
    dict_obj! {py !
        type_name: "JoinGroupRequest",
        seq: event.msg_seq,
        time: datetime_from_ts(py, event.msg_time).map(|v| v.into_py(py))?,
        group_uin: event.group_code,
        group_name: event.group_name,
        request_uin: event.req_uin,
        request_nickname: event.req_nick,
        suspicious: event.suspicious,
        invitor_uin: event.invitor_uin,
        invitor_nickname: event.invitor_nick,
    }
}

fn handle_group_invitation(event: rce::SelfInvitedEvent) -> PyRet {
    let event = event.inner;
    dict_obj! {py !
        type_name: "JoinGroupInvitation",
        seq: event.msg_seq,
        time: datetime_from_ts(py, event.msg_time).map(|v| v.into_py(py))?,
        group_uin: event.group_code,
        group_name: event.group_name,
        invitor_uin: event.invitor_uin,
        invitor_nickname: event.invitor_nick,
    }
}

fn handle_friend_request(event: rce::NewFriendRequestEvent) -> PyRet {
    let event = event.inner;
    dict_obj! {
        type_name: "NewFriendRequest",
        seq: event.msg_seq,
        uin: event.req_uin,
        nickname: event.req_nick,
        message: event.message,
    }
}
