from __future__ import annotations

import asyncio
from contextvars import ContextVar
from functools import partial
from typing import Any, Awaitable, Literal, Optional, Protocol, Set
from typing_extensions import Literal, Self

from graia.broadcast import Broadcast
from graia.broadcast.entities.dispatcher import BaseDispatcher
from graia.broadcast.interfaces.dispatcher import DispatcherInterface as DI
from launart import Launart, Launchable
from loguru import logger

from ichika import core
from ichika.client import Client
from ichika.login import BaseLoginCredentialStore, login_password, login_qrcode
from ichika.login.password import PasswordLoginCallbacks
from ichika.login.qrcode import QRCodeLoginCallbacks
from ichika.utils import generic_issubclass

BROADCAST_EVENT = ContextVar("ICHIKA_BROADCAST_EVENT")
CLIENT_INSTANCE = ContextVar("ICHIKA_CLIENT_INSTANCE")


class IchikaClientDispatcher(BaseDispatcher):
    @staticmethod
    async def catch(interface: DI):
        if generic_issubclass(Client, interface.annotation):
            return CLIENT_INSTANCE.get()


class BroadcastCallback:
    broadcast: Broadcast

    def __init__(self, broadcast: Optional[Broadcast] = None) -> None:
        loop = asyncio.get_running_loop()
        if not broadcast:
            broadcast = Broadcast(loop=loop)
        if broadcast.loop is not loop:
            raise ValueError("Graia Broadcast had a different event loop!")
        self.broadcast = broadcast
        if IchikaClientDispatcher not in broadcast.prelude_dispatchers:
            broadcast.prelude_dispatchers.append(IchikaClientDispatcher)

    async def put(self, data: Any) -> None:
        from .event import EVENT_TYPES

        client = data.pop("client")

        e = EVENT_TYPES[data.pop("type_name")](**data)
        client_token = CLIENT_INSTANCE.set(client)
        event_token = BROADCAST_EVENT.set(e)
        await self.broadcast.postEvent(e)
        BROADCAST_EVENT.reset(event_token)
        CLIENT_INSTANCE.reset(client_token)


class IchikaComponent(Launchable):
    class _LoginPartial(Protocol):
        def __call__(
            self,
            *,
            store: BaseLoginCredentialStore,
            event_callbacks: list[core.EventCallback],
        ) -> Awaitable[Client]:
            ...

    def __init__(self, store: BaseLoginCredentialStore, broadcast: Optional[Broadcast] = None) -> None:
        self.broadcast = broadcast
        self.store: BaseLoginCredentialStore = store
        self.login_partials: dict[int, IchikaComponent._LoginPartial] = {}
        self.client_hb_map: dict[int, tuple[Client, Awaitable[None]]] = {}
        super().__init__()

    id = "ichika.main"

    @property
    def stages(self) -> Set[Literal["preparing", "blocking", "cleanup"]]:
        return {"preparing", "blocking", "cleanup"}

    @property
    def required(self) -> Set[str]:
        return set()

    def add_password_login(
        self,
        uin: int,
        credential: str | bytes,
        /,
        protocol: str = "AndroidPad",
        callbacks: PasswordLoginCallbacks | None = None,
        use_sms: bool = True,
    ) -> Self:
        if uin in self.login_partials:
            raise ValueError(f"uin {uin} already exists")
        self.login_partials[uin] = partial(
            login_password,
            uin,
            credential,
            protocol=protocol,
            login_callbacks=callbacks,
            use_sms=use_sms,
        )
        return self

    def add_qrcode_login(
        self,
        uin: int,
        /,
        protocol: Literal["AndroidWatch"] = "AndroidWatch",
        callbacks: QRCodeLoginCallbacks | None = None,
    ) -> Self:
        if uin in self.login_partials:
            raise ValueError(f"uin {uin} already exists")
        self.login_partials[uin] = partial(login_qrcode, uin, protocol=protocol, login_callbacks=callbacks)
        return self

    async def launch(self, mgr: Launart):
        if self.broadcast is None:
            self.broadcast = Broadcast(loop=asyncio.get_running_loop())
        elif self.broadcast.loop is not asyncio.get_running_loop():
            raise ValueError("Graia Broadcast had a different event loop!")
        broadcast_cb = BroadcastCallback(self.broadcast)
        async with self.stage("preparing"):
            for uin, login_fn in self.login_partials.items():
                try:
                    logger.info(f"Trying to login: {uin}")
                    client = await login_fn(store=self.store, event_callbacks=[broadcast_cb])
                    self.client_hb_map[uin] = (client, client.keep_alive())
                except Exception as e:
                    logger.exception(f"Login failed: {uin}", e)

        async with self.stage("blocking"):
            await mgr.status.wait_for_sigexit()

        async with self.stage("cleanup"):
            for uin, (client, hb) in self.client_hb_map.items():
                logger.info(f"Stopping client: {uin}")
                await client.stop()
                await hb
                logger.success(f"Client {uin} stopped")
