# 更新日志

本文件使用 [Keep a Changelog](https://keepachangelog.com/zh-CN) 格式。

本项目使用 [towncrier](https://towncrier.readthedocs.io) 作为更新日志生成器。
所有处理的问题可在 [GitHub Issues](https://github.com/BlueGlassBlock/Ichika/issues) 找到。

<!-- towncrier release notes start -->

## [0.0.6](https://github.com/BlueGlassBlock/ichika/tree/0.0.6) - 2023-05-13

你可以在 [PyPI](https://pypi.org/project/ichika/0.0.6/) 找到该版本。

### 新增

- 使用 [`backon`](https://docs.rs/backon) 提供自动重试。 ([#55](https://github.com/BlueGlassBlock/ichika/issues/55))
- `Member.card_name` 现在表示合并后的名片。原始名片存储于 `Member.raw_card_name` 中。 ([#56](https://github.com/BlueGlassBlock/ichika/issues/56))
- 支持处理群名更新事件。
- 添加 `Client.get_profile` 以获取用户公开资料。
- 添加了获取群员列表的方法。


### 更改

- 优化了首次登录即失败以及退出时掉线的逻辑。 ([#57](https://github.com/BlueGlassBlock/ichika/issues/57))
- `Client.get_group_admins` 的返回类型更改为 `list[Member]`。 ([#65](https://github.com/BlueGlassBlock/ichika/issues/65))
- 使用 `Enum` 表示性别和权限。 ([#68](https://github.com/BlueGlassBlock/ichika/issues/68))
- 使用 `Literal` 标注了可用密码登录的协议列表。
- 更改了 Rust 侧日志的显示风格。
- 现在自动重连将采取最小 3s，最大 60s，每次增长 1.2 倍的间隔时间，并不再主动停止重试。
- 现在要使用刷新缓存的 API 应传入 `cache = False` 而不是调用 `get_xxx_raw` 方法。
- 设定每个账号的群员和群的缓存大小为 1024。
- 重命名 `ichika.core.Profile.sex` 为 `ichika.core.Profile.gender`。
- 默认限制使用 4 个线程进行操作。你可以通过 `ICHIKA_RUNTIME_THREAD_COUNT` 环境变量来修改这个限制。


### 修复

- 修复了事件无法正确在 Union 中分发的 bug。 ([#58](https://github.com/BlueGlassBlock/ichika/issues/58))
- 修复 `At` 的 `target` 属性发送时被忽略的问题。 ([#59](https://github.com/BlueGlassBlock/ichika/issues/59))
- 修复了 `GroupMute` 在 Rust 端提供参数名不吻合的问题。 ([#60](https://github.com/BlueGlassBlock/ichika/issues/60))
- 修复了 `IchikaComponent` 在 cleanup 阶段分发事件导致的错误。 ([#61](https://github.com/BlueGlassBlock/ichika/issues/61))
- 客户端注册失败现在会直接报错。 ([#67](https://github.com/BlueGlassBlock/ichika/issues/67))
- 修复了因网络原因掉线时，无法多次重试的问题。 ([#69](https://github.com/BlueGlassBlock/ichika/issues/69))
- 修复了事件的属性无法被类型检查器正常识别的问题。


## [0.0.5](https://github.com/BlueGlassBlock/ichika/tree/0.0.5) - 2023-05-03

你可以在 [PyPI](https://pypi.org/project/ichika/0.0.5/) 找到该版本。

### 新增

- 增加了适用于 `Launart` 的 `IchikaComponent` 可启动组件。
- 支持上传与发送音频。
- 支持发送和接收“回复”元素。请注意该元素和图片一起使用时可能发生 bug。
- 支持处理“请求”事件（好友申请、加群申请、入群邀请）。
- 支持处理全体禁言和群员禁言事件。
- 支持处理其他群员退群事件。
- 支持处理删除好友事件（无论是主动还是被动）。
- 支持处理新增好友事件。
- 支持处理新成员进群事件。
- 支持处理群员权限更新事件。
- 支持处理群解散事件。
- 支持接收、下载和上传转发消息。
- 支持接收和发送音乐分享。
- 支持接收好友申请、加群申请与被邀请入群事件。
- 添加了 `Android Pad` 协议。
- 添加了基础的 [`Graia Project`](https://github.com/GraiaProject) 绑定。


### 更改

- 使用异步登录回调。 ([#25](https://github.com/BlueGlassBlock/ichika/issues/25))
- 群组事件的 `Group` 对象不再挂靠于 `MemberInfo`，而是存储于 `Group` 属性。 ([#29](https://github.com/BlueGlassBlock/ichika/issues/29))
- 使用 `dict` 作为事件传递结构以方便其他框架绑定。 ([#34](https://github.com/BlueGlassBlock/ichika/issues/34))
- 使用 `str` 作为 `protocol` 值，并同步所有协议至最新版本。
- 更改了构建信息的键名。


### 修复

- 暂时删除了来自 RICQ 的无用 `LoginEvent` 以避免启动时的报错。


### 其他

- 升级 [`syn`](https://github.com/dtolnay/syn) 至 `2.x.x`。


## [0.0.4](https://github.com/BlueGlassBlock/ichika/tree/0.0.4) - 2023-03-17

你可以在 [PyPI](https://pypi.org/project/ichika/0.0.4/) 找到该版本。

### 新增

- 支持处理群聊和好友撤回消息事件 ([#22](https://github.com/BlueGlassBlock/ichika/issues/22))
- 修复了消息元素的 `__repr__` 显示。
- 支持好友和群组的拍一拍（双击头像触发）事件。


### 更改

- 使用 `asyncio.Queue` 而不是回调函数来处理事件。

  `Queue.put` 的任务上下文会与 `ichika.login.xxx_login` 的调用者一致。


## [0.0.3](https://github.com/BlueGlassBlock/ichika/tree/0.0.3) - 2023-03-16

你可以在 [PyPI](https://pypi.org/project/ichika/0.0.3/) 找到该版本。



### 新增

- 支持以下 API:
  - 发送消息
  - 拍一拍
  - 撤回消息
  - 获取群信息
  - 获取好友列表
  - 获取群员信息
  - 获取好友信息
  - 获取自身信息
  - 修改名片
  - 查询群管理员
  - 修改群员信息
  - 修改群员权限
  - 修改群信息
  - 群聊打卡
  - 修改自身信息
  - 修改在线状态
  - 退出群聊
  - 设置群精华
  - 踢出群员
  - 删除好友
  - 禁言
  - 取消禁言
  - 全体禁言
  - 取消全体禁言

### 其他

- 使用 [`towncrier`](https://towncrier.readthedocs.io) 和 GitHub Release 来管理项目。 ([#18](https://github.com/BlueGlassBlock/ichika/issues/18))
