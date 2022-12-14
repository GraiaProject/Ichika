from __future__ import annotations


class SealedMarketFace:
    name: str


class SealedImage:
    md5: bytes
    size: int
    width: int
    height: int
    image_type: int
