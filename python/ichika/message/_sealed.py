from __future__ import annotations


class SealedMarketFace:  # Rust Native
    name: str


class SealedImage:  # Rust Native
    md5: bytes
    size: int
    width: int
    height: int
    image_type: int


class SealedAudio:  # Rust Native
    md5: bytes
    size: int
    file_type: int
