use pyo3::prelude::*;
use pyo3::types::PyDict;
use ricq::client::event as rce;
use ricq::handler::QEvent;

use super::MessageSource;
use crate::client::structs::Friend;
use crate::dict_obj;
use crate::exc::MapPyErr;
use crate::message::convert::{serialize_as_py_chain, serialize_audio};
use crate::utils::{datetime_from_ts, py_try, timedelta_from_secs};

type PyDictRet = PyResult<Py<PyDict>>;

pub async fn convert(event: QEvent) -> PyDictRet {
    match event {
        QEvent::Login(_) => dict_obj! {},
        QEvent::GroupMessage(event) => handle_group_message(event).await,
        QEvent::GroupAudioMessage(event) => handle_group_audio(event).await,
        QEvent::FriendMessage(event) => handle_friend_message(event).await,
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

async fn handle_group_message(event: rce::GroupMessageEvent) -> PyDictRet {
    let msg = event.inner;

    let content = py_try(|py| serialize_as_py_chain(py, msg.elements))?;
    dict_obj! {py !
        type_name: "GroupMessage",
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
        content: content,
        group: msg.group_code,
        sender: msg.from_uin,
    }
}

async fn handle_group_recall(event: rce::GroupMessageRecallEvent) -> PyDictRet {
    let event = event.inner;
    let time = py_try(|py| Ok(datetime_from_ts(py, event.time)?.into_py(py)))?;
    dict_obj! {
        type_name: "GroupRecallMessage",
        time: time,
        group: event.group_code,
        author: event.author_uin,
        operator: event.operator_uin,
        seq: event.msg_seq,
    }
}

async fn handle_group_audio(event: rce::GroupAudioMessageEvent) -> PyDictRet {
    let url = event.url().await.py_res()?;
    let msg = event.inner;
    let content = py_try(|py| serialize_audio(py, url, &msg.audio.0))?;
    dict_obj! {py !
        type_name: "GroupMessage",
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
        content: content,
        group: msg.group_code,
        sender: msg.from_uin,
    }
}

async fn handle_friend_message(event: rce::FriendMessageEvent) -> PyDictRet {
    let msg = event.inner;
    let content = py_try(|py| serialize_as_py_chain(py, msg.elements))?;
    dict_obj! {py !
        type_name: "FriendMessage",
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
        content: content,
        sender: msg.from_uin,
    }
}

async fn handle_friend_recall(event: rce::FriendMessageRecallEvent) -> PyDictRet {
    let event = event.inner;
    let time = py_try(|py| Ok(datetime_from_ts(py, event.time)?.into_py(py)))?;
    dict_obj! {
        type_name: "FriendRecallMessage",
        time: time,
        author: event.friend_uin,
        seq: event.msg_seq,
    }
}

async fn handle_friend_audio(event: rce::FriendAudioMessageEvent) -> PyDictRet {
    let url = event.url().await.py_res()?;
    let msg = event.inner;
    let content = py_try(|py| serialize_audio(py, url, &msg.audio.0))?;
    dict_obj! {py !
        type_name: "FriendMessage",
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
        content: content,
        sender: msg.from_uin,
    }
}

async fn handle_temp_message(event: rce::GroupTempMessageEvent) -> PyDictRet {
    let msg = event.inner;
    let content = py_try(|py| serialize_as_py_chain(py, msg.elements))?;


    dict_obj! {py !
        type_name: "TempMessage",
        source: MessageSource::new(py, &msg.seqs, &msg.rands, msg.time)?,
        content: content,
        group: msg.group_code,
        sender: msg.from_uin,
    }
}

async fn handle_group_nudge(event: rce::GroupPokeEvent) -> PyDictRet {
    let event = event.inner;

    dict_obj! {
        type_name: "GroupNudge",
        group: event.group_code,
        sender: event.sender,
        receiver: event.receiver,
    }
}

async fn handle_friend_nudge(event: rce::FriendPokeEvent) -> PyDictRet {
    let client = event.client;
    if client.uin().await == event.inner.sender {
        return dict_obj! {};
    }
    let event = event.inner;
    dict_obj! {
        type_name: "FriendNudge",
        sender: event.sender,
    }
}

fn handle_new_friend(event: rce::NewFriendEvent) -> PyDictRet {
    let friend: Friend = event.inner.into();
    dict_obj! {
        type_name: "NewFriend",
        friend: friend,
    }
}
async fn handle_new_member(event: rce::NewMemberEvent) -> PyDictRet {
    let event = event.inner;
    dict_obj! {
        type_name: "NewMember",
        group: event.group_code,
        member: event.member_uin,
    }
}

async fn handle_group_leave(event: rce::GroupLeaveEvent) -> PyDictRet {
    let event = event.inner;

    dict_obj! {
        type_name: "MemberLeaveGroup",
        group_uin: event.group_code,
        member_uin: event.member_uin,
    }
}

async fn handle_group_disband(event: rce::GroupDisbandEvent) -> PyDictRet {
    let event = event.inner;
    dict_obj! {
        type_name: "GroupDisband",
        group_uin: event.group_code,
        operator_uin: event.operator_uin,
    }
}

async fn handle_friend_delete(event: rce::DeleteFriendEvent) -> PyDictRet {
    dict_obj! {
        type_name: "FriendDeleted",
        friend_uin: event.inner.uin,
    }
}

async fn handle_mute(event: rce::GroupMuteEvent) -> PyDictRet {
    let event = event.inner;

    if event.target_uin == 0 {
        return dict_obj! {
            type_name: "GroupMute",
            group: event.group_code,
            operator: event.operator_uin,
            status: event.duration.as_secs() == 0
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
    dict_obj! {
        type_name: "MemberMute",
        group: event.group_code,
        operator: event.operator_uin,
        target: event.target_uin,
        duration: duration,
    }
}

async fn handle_permission_change(event: rce::MemberPermissionChangeEvent) -> PyDictRet {
    let event = event.inner;
    dict_obj! {
        type_name: "MemberPermissionChange",
        group: event.group_code,
        target: event.member_uin,
        permission: event.new_permission as u8,
    }
}

async fn handle_group_info_update(event: rce::GroupNameUpdateEvent) -> PyDictRet {
    let event = event.inner;
    let info: Py<PyDict> = dict_obj! {
        name: event.group_name
    }?;
    dict_obj! {
        type_name: "GroupInfoUpdate",
        group: event.group_code,
        operator: event.operator_uin,
        info: info,
    }
}

fn handle_group_request(event: rce::JoinGroupRequestEvent) -> PyDictRet {
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

fn handle_group_invitation(event: rce::SelfInvitedEvent) -> PyDictRet {
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

fn handle_friend_request(event: rce::NewFriendRequestEvent) -> PyDictRet {
    let event = event.inner;
    dict_obj! {
        type_name: "NewFriendRequest",
        seq: event.msg_seq,
        uin: event.req_uin,
        nickname: event.req_nick,
        message: event.message,
    }
}
