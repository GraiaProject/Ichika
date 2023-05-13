from dataclasses import dataclass
from datetime import datetime
from typing import Awaitable, Literal, Protocol, Sequence, TypeVar, type_check_only
from typing_extensions import Any, TypeAlias

from ichika.message.elements import MusicShare
from ichika.structs import Gender, GroupPermission

from . import event_defs as event_defs
from .client import Client, HttpClientProto
from .login import (
    BaseLoginCredentialStore,
    PasswordLoginCallbacks,
    QRCodeLoginCallbacks,
)
from .message._sealed import SealedAudio

__version__: str
__build__: Any

_T_Event: TypeAlias = event_defs._T_Event

@type_check_only
class EventCallback(Protocol):
    """描述事件处理回调的协议，被刻意设计为与 [`asyncio.Queue`][asyncio.Queue] 兼容。"""

    async def put(self, event: _T_Event, /) -> Any:
        """处理事件"""

# Here, outside wrapper "login_XXX" ensures that a "task locals" can be acquired for event task execution.

async def password_login(
    uin: int,
    credential: str | bytes,
    use_sms: bool,
    protocol: str,
    store: BaseLoginCredentialStore,
    event_callbacks: Sequence[EventCallback],
    login_callbacks: PasswordLoginCallbacks,
) -> Client:
    """使用密码登录。

    :param uin: 账号
    :param credential: 登录凭据，str 为密码，bytes 为 MD5 数据
    :param use_sms: 是否使用短信验证码验证设备锁
    :param protocol: 登录协议，为 AndroidPhone, AndroidPad, AndroidWatch, IPad, MacOS, QiDian 中的一个
    :param store: 登录凭据存储器
    :param event_callbacks: 事件队列
    :param login_callbacks: 用于解析登录的回调
    :return: 可操作的客户端
    """

async def qrcode_login(
    uin: int,
    protocol: str,
    store: BaseLoginCredentialStore,
    event_callbacks: Sequence[EventCallback],
    login_callbacks: QRCodeLoginCallbacks,
) -> Client:
    """使用二维码登录。

    :param uin: 账号
    :param protocol: 登录协议，只能使用 AndroidWatch
    :param store: 登录凭据存储器
    :param event_callbacks: 事件队列
    :param login_callbacks: 用于解析登录的回调
    :return: 可操作的客户端
    """

# region: client

_internal_repr = dataclass(frozen=True, init=False)

@_internal_repr
class AccountInfo:
    """机器人账号信息"""

    nickname: str
    """机器人昵称"""
    age: int
    """机器人年龄"""
    gender: Gender
    """机器人标注的性别"""

@_internal_repr
class OtherClientInfo:
    """获取到的其他客户端信息"""

    app_id: int
    """应用 ID"""
    instance_id: int
    """实例 ID"""
    sub_platform: str
    """子平台"""
    device_kind: str
    """设备类型"""

@_internal_repr
class Friend:
    """好友信息"""

    uin: int
    """账号 ID"""
    nick: str
    """好友昵称"""
    remark: str
    """好友备注"""
    face_id: int
    """未知"""
    group_id: int
    """好友分组 ID"""

@_internal_repr
class FriendGroup:
    """好友组"""

    group_id: int
    """分组 ID"""
    name: str
    """组名"""
    total_count: int
    """组内总好友数"""
    online_count: int
    """组内在线好友数"""
    seq_id: int
    """SEQ ID"""

@_internal_repr
class FriendList:
    """好友列表，你通过 API 获取到的顶层数据结构"""

    total_count: int
    """所有好友数"""
    online_count: int
    """在线好友数"""
    def friends(self) -> tuple[Friend,]:
        """获取好友列表。

        :return: 好友列表
        """
    def find_friend(self, uin: int) -> Friend | None:
        """查找好友。

        :param uin: 好友账号
        :return: 好友信息
        """
    def friend_groups(self) -> tuple[FriendGroup,]:
        """获取好友分组列表。

        :return: 好友分组列表
        """
    def find_friend_group(self, group_id: int) -> FriendGroup | None:
        """查找好友分组。

        :param group_id: 好友分组 ID
        :return: 好友分组信息
        """

@_internal_repr
class Profile:
    """描述账号资料"""

    uin: int
    """账号"""
    gender: Gender
    """性别"""
    age: int
    """年龄"""
    nickname: str
    """设置的昵称"""
    level: int
    """等级"""
    city: str
    """设置的城市"""
    sign: str
    """设置的个性签名"""
    login_days: int
    """连续登录天数"""

@_internal_repr
class Group:
    """群组信息，请注意通过缓存获取的数据可能不精确"""

    uin: int
    """群号"""
    name: str
    """群名"""
    memo: str
    """群公告"""
    owner_uin: int
    """群主账号"""
    create_time: int
    """群创建时间戳"""
    level: int
    """群等级"""
    member_count: int
    """群成员数量"""
    max_member_count: int
    """群最大成员数量"""
    global_mute_timestamp: int
    """全局禁言时间戳"""
    mute_timestamp: int
    """群禁言时间戳"""
    last_msg_seq: int
    """最后一条消息序列号"""

@_internal_repr
class Member:
    """群成员信息"""

    group_uin: int
    """群号"""
    uin: int
    """账号"""
    gender: Gender
    """性别"""
    nickname: str
    """昵称"""
    card_name: str
    """群名片"""
    level: int
    """成员等级"""
    join_time: int
    """加入时间"""
    last_speak_time: int
    """最后发言时间"""
    special_title: str
    """特殊头衔"""
    special_title_expire_time: int
    """特殊头衔过期时间"""
    mute_timestamp: int
    """禁言时间戳"""
    permission: GroupPermission
    """权限"""

_T = TypeVar("_T")

VTuple = tuple[_T,]

@_internal_repr
class RawMessageReceipt:
    seq: int
    rand: int
    raw_seqs: VTuple[int]
    """消息 SEQ ID"""
    rwa_rands: VTuple[int]
    """消息随机数"""
    time: int
    """发送时间戳"""
    kind: str
    """消息类型，为 `group` 与 `friend` 中一个"""
    target: int
    """发送目标"""

@_internal_repr
class OCRText:
    """单条 OCR 结果"""

    detected_text: str
    """识别出的文本"""
    confidence: int
    """置信度"""
    polygon: VTuple[tuple[int, int]] | None
    """文本所在区域的顶点坐标"""
    advanced_info: str
    """额外信息"""

@_internal_repr
class OCRResult:
    """OCR 结果"""

    texts: VTuple[OCRText]
    """识别出的文本列表"""
    language: str
    """语言"""

__OnlineStatus: TypeAlias = (  # TODO: Wrapper
    tuple[int, str]  # (face_index, wording)
    | tuple[
        Literal[False],
        Literal[
            11,  # 在线
            21,  # 离线，效果未知
            31,  # 离开
            41,  # 隐身
            50,  # 忙
            60,  # Q 我吧
            70,  # 请勿打扰
        ],
    ]
    | tuple[
        Literal[True],
        Literal[
            1000,  # 当前电量
            1028,  # 听歌中
            1040,  # 星座运势
            1030,  # 今日天气
            1069,  # 遇见春天
            1027,  # Timi中
            1064,  # 吃鸡中
            1051,  # 恋爱中
            1053,  # 汪汪汪
            1019,  # 干饭中
            1018,  # 学习中
            1032,  # 熬夜中
            1050,  # 打球中
            1011,  # 信号弱
            1024,  # 在线学习
            1017,  # 游戏中
            1022,  # 度假中
            1021,  # 追剧中
            1020,  # 健身中
        ],
    ]
)

class PlumbingClient:
    """Ichika 的底层客户端，暴露了一些底层接口"""

    # [impl 1]
    @property
    def uin(self) -> int:
        """当前登录的账号的 QQ 号。"""
    @property
    def online(self) -> bool:
        """当前账号是否登录成功。"""
    def keep_alive(self) -> Awaitable[None]:
        """保持在线。

        :return: 承载了维持心跳和重连任务的 [`Future 对象`][asyncio.Future]。
        """
    async def stop(self) -> None:
        """停止客户端运行。

        请在本方法返回后再等待 [`keep_alive`][ichika.core.PlumbingClient.keep_alive] 方法返回的 [`Future 对象`][asyncio.Future]。
        """
    async def get_profile(self, uin: int) -> Profile:
        """获取任意账号的资料

        :param uin: 账号
        :return: 对应账号的资料
        """
    async def get_account_info(self) -> AccountInfo:
        """获取当前登录的账号的信息。

        :return: 当前登录的账号的信息
        """
    async def set_account_info(
        self,
        *,
        name: str | None = None,
        email: str | None = None,
        personal_note: str | None = None,
        company: str | None = None,
        college: str | None = None,
        signature: str | None = None,
    ) -> None:
        """设置当前登录的账号的信息。

        :param name: 昵称，None 为不修改
        :param email: 邮箱，None 为不修改
        :param personal_note: 个人说明，None 为不修改
        :param company: 公司，None 为不修改
        :param college: 学校，None 为不修改
        :param signature: 个性签名，None 为不修改
        """
    async def get_other_clients(self) -> VTuple[OtherClientInfo]:
        """获取其他在线客户端的信息。

        :return: 一个元组，包含其他在线客户端的信息
        """
    async def modify_online_status(self, status: __OnlineStatus) -> None:
        """修改当前登录的账号的在线状态。

        :param status: 在线状态
        """
    async def image_ocr(self, url: str, md5: str, width: int, height: int) -> OCRResult:
        """对图片进行 OCR 识别。

        :param url: 图片 URL
        :param md5: 图片 MD5
        :param width: 图片宽度
        :param height: 图片高度
        :return: OCR 结果
        """
    # [impl 2]
    async def get_friend_list(self, cache: bool = True) -> FriendList:
        """获取好友列表。

        :param cache: 是否使用缓存
        :return: 好友列表
        """
    async def get_friends(self) -> VTuple[Friend]:
        """获取好友列表。

        :return: 好友列表
        """
    async def find_friend(self, uin: int) -> Friend | None:
        """查找好友。

        :param uin: 好友 QQ 号
        :return: 好友对象，如果不存在则返回 None
        """
    async def nudge_friend(self, uin: int) -> None:
        """给好友发送窗口抖动。

        :param uin: 好友 QQ 号
        """
    async def delete_friend(self, uin: int) -> None:
        """删除好友。

        :param uin: 好友 QQ 号
        """
    # [impl 3]
    async def get_group(self, uin: int, cache: bool = True) -> Group:
        """获取群信息。

        :param uin: 群号
        :param cache: 是否使用缓存
        :return: 群信息
        """
    async def find_group(self, uin: int) -> Group | None:
        """查找群。

        :param uin: 群号
        :return: 群对象，如果不存在则返回 None
        """
    async def get_groups(self) -> VTuple[Group]:
        """获取群列表。

        :return: 群列表
        """
    async def get_group_admins(self, uin: int) -> list[Member]:
        """获取群管理员列表。

        :param uin: 群号
        :return: 群管理员列表
        """
    async def mute_group(self, uin: int, mute: bool) -> None:
        """禁言/解禁群成员。

        :param uin: 群号
        :param mute: 是否禁言
        """
    async def quit_group(self, uin: int) -> None:
        """退出群。

        :param uin: 群号
        """
    async def modify_group_info(self, uin: int, *, memo: str | None = None, name: str | None = None) -> None:
        """修改群信息。

        :param uin: 群号
        :param memo: 群公告
        :param name: 群名称
        """
    async def group_sign_in(self, uin: int) -> None:
        """签到群。

        :param uin: 群号
        """
    # [impl 4]
    async def get_member(self, group_uin: int, uin: int, cache: bool = False) -> Member:
        """获取群成员信息。

        :param group_uin: 群号
        :param uin: QQ 号
        :param cache: 是否使用缓存
        :return: 群成员信息
        """
    async def get_member_list(self, group_uin: int, cache: bool = True) -> list[Member]:
        """获取群成员列表。

        :param group_uin: 群号
        :param cache: 是否使用缓存
        :return: 群成员列表
        """
    async def nudge_member(self, group_uin: int, uin: int) -> None:
        """给群成员发送窗口抖动。

        :param group_uin: 群号
        :param uin: QQ 号
        """
    # Duration -> 0: Unmute
    async def mute_member(self, group_uin: int, uin: int, duration: int) -> None:
        """禁言/解禁群成员。

        :param group_uin: 群号
        :param uin: QQ 号
        :param duration: 禁言时长，单位为秒，0 表示解禁
        """
    async def kick_member(self, group_uin: int, uin: int, msg: str, block: bool) -> None:
        """踢出群成员。

        :param group_uin: 群号
        :param uin: QQ 号
        :param msg: 踢人理由
        :param block: 是否加入黑名单
        """
    async def modify_member_special_title(self, group_uin: int, uin: int, special_title: str) -> None:
        """修改群成员专属头衔。

        :param group_uin: 群号
        :param uin: QQ 号
        :param special_title: 专属头衔
        """
    async def modify_member_card(self, group_uin: int, uin: int, card_name: str) -> None:
        """修改群成员名片。

        :param group_uin: 群号
        :param uin: QQ 号
        :param card_name: 名片
        """
    async def modify_member_admin(self, group_uin: int, uin: int, admin: bool) -> None:
        """设置/取消群管理员。

        :param group_uin: 群号
        :param uin: QQ 号
        :param admin: 是否设置为管理员
        """
    # [impl 5]
    async def upload_friend_image(self, uin: int, data: bytes) -> dict[str, Any]:
        """上传好友图片。

        :param uin: QQ 号
        :param data: 图片数据
        :return: 上传结果
        """
    async def upload_friend_audio(self, uin: int, data: bytes) -> dict[str, Any]:
        """上传好友语音。

        :param uin: QQ 号
        :param data: 语音数据
        :return: 上传结果
        """
    async def upload_group_image(self, uin: int, data: bytes) -> dict[str, Any]:
        """上传群图片。

        :param uin: QQ 号
        :param data: 图片数据
        :return: 上传结果
        """
    async def upload_group_audio(self, uin: int, data: bytes) -> dict[str, Any]:
        """上传群语音。

        :param uin: QQ 号
        :param data: 语音数据
        :return: 上传结果
        """
    async def send_friend_audio(self, uin: int, audio: SealedAudio) -> RawMessageReceipt:
        """发送好友语音。

        :param uin: QQ 号
        :param audio: 语音数据
        :return: 发送结果
        """
    async def send_group_audio(self, uin: int, audio: SealedAudio) -> RawMessageReceipt:
        """发送群语音。

        :param uin: QQ 号
        :param audio: 语音数据
        :return: 发送结果
        """
    async def send_friend_music_share(self, uin: int, share: MusicShare) -> RawMessageReceipt:
        """发送好友音乐分享。

        :param uin: QQ 号
        :param share: 音乐分享信息
        :return: 发送结果
        """
    async def send_group_music_share(self, uin: int, share: MusicShare) -> RawMessageReceipt:
        """发送群音乐分享。

        :param uin: QQ 号
        :param share: 音乐分享信息
        :return: 发送结果
        """
    async def download_forward_msg(self, downloader: HttpClientProto, res_id: str) -> list[dict]:
        """下载转发消息。

        :param downloader: 下载器
        :param res_id: 资源 ID
        :return: 转发消息
        """
    async def upload_forward_msg(self, group_uin: int, msg: list[dict]) -> tuple[str, str, str]:
        """上传转发消息。

        :param group_uin: 群号
        :param msg: 转发消息
        :return: 上传结果
        """
    # [impl 6]
    async def send_friend_message(self, uin: int, chain: list[dict[str, Any]]) -> RawMessageReceipt:
        """发送好友消息。

        :param uin: QQ 号
        :param chain: 消息链
        :return: 发送结果
        """
    async def send_group_message(self, uin: int, chain: list[dict[str, Any]]) -> RawMessageReceipt:
        """发送群消息。

        :param uin: QQ 号
        :param chain: 消息链
        :return: 发送结果
        """
    async def recall_friend_message(self, uin: int, time: int, seq: int, rand: int) -> None:
        """撤回好友消息。

        :param uin: QQ 号
        :param time: 消息发送时间
        :param seq: 消息的 SEQ
        :param rand: 消息的随机序列号
        """
    async def recall_group_message(self, uin: int, seq: int, rand: int) -> None:
        """撤回群消息。

        :param uin: QQ 号
        :param seq: 消息的 SEQ
        :param rand: 消息的随机序列号
        """
    async def modify_group_essence(self, uin: int, seq: int, rand: int, flag: bool) -> None:
        """修改群消息精华状态。

        :param uin: QQ 号
        :param seq: 消息的 SEQ
        :param rand: 消息的随机序列号
        :param flag: 是否设为精华
        """
    # [impl 7]
    async def process_join_group_request(
        self, seq: int, request_uin: int, group_uin: int, accept: bool, block: bool, message: str
    ) -> None:
        """
        处理加群请求。

        :param seq: 消息的 SEQ
        :param request_uin: 请求人 QQ 号
        :param group_uin: 群号
        :param accept: 是否同意
        :param block: 是否拒绝并加入黑名单
        :param message: 回复消息
        """
    async def process_group_invitation(self, seq: int, invitor_uin: int, group_uin: int, accept: bool) -> None:
        """
        处理群邀请。

        :param seq: 消息的 SEQ
        :param invitor_uin: 邀请人 QQ 号
        :param group_uin: 群号
        :param accept: 是否同意
        """
    async def process_new_friend_request(self, seq: int, request_uin: int, accept: bool) -> None:
        """
        处理加好友请求。

        :param seq: 消息的 SEQ
        :param request_uin: 请求人 QQ 号
        :param accept: 是否同意
        """

# endregion: client

def face_id_from_name(name: str) -> int | None: ...
def face_name_from_id(id: int) -> str: ...
@_internal_repr
class MessageSource:
    """消息元信息"""

    seq: int
    """消息的 SEQ

    建议搭配聊天类型与上下文 ID （例如 `("group", 123456, seq)`）作为索引的键
    """
    rand: int
    """消息的随机序列号，撤回需要"""

    raw_seqs: VTuple[int]
    """消息的原始 SEQ"""

    raw_rands: VTuple[int]
    """消息的原始随机序列号"""

    time: datetime
    """消息发送时间"""
