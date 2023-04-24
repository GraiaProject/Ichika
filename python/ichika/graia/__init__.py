from __future__ import annotations

import asyncio
from contextvars import ContextVar
from typing import Optional

from graia.broadcast import Broadcast
from graia.broadcast.entities.dispatcher import BaseDispatcher
from graia.broadcast.interfaces.dispatcher import DispatcherInterface as DI

from ichika.client import Client
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

    async def put(self, data: dict):
        from .event import EVENT_TYPES

        client = data.pop("client")

        e = EVENT_TYPES[data.pop("type_name")](**data)
        client_token = CLIENT_INSTANCE.set(client)
        event_token = BROADCAST_EVENT.set(e)
        await self.broadcast.postEvent(e)
        BROADCAST_EVENT.reset(event_token)
        CLIENT_INSTANCE.reset(client_token)
