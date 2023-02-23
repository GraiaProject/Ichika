from __future__ import annotations


class Dense1x2:
    chars: list[str] = [" ", "\u2584", "\u2580", "\u2588"]

    def __init__(self, invert: bool = False) -> None:
        self.inv = invert

    def render(self, data: list[list[bool]]) -> str:
        if not data:
            return ""
        block: list[str] = [
            "".join(
                self.chars[(upper ^ self.inv) * 2 + (lower ^ self.inv)] for upper, lower in zip(data[i], data[i + 1])
            )
            for i in range(0, len(data) - 1, 2)
        ]
        if len(data) % 2 != 0:
            block.append("".join(self.chars[(pixel ^ self.inv) * 2] for pixel in data[-1]))
        return "\n".join(block)


if __name__ == "__main__":
    print(Dense1x2(True).render([[True, False], [False, True], [True, False]]))
