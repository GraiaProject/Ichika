from graia.amnesia.message import Element
from enum import Enum
from dataclasses import dataclass
from typing import Literal
from typing_extensions import Self
from functools import total_ordering
from graia.amnesia.message.element import Text as Text


@dataclass
class At(Element):
    target: int
    display: str | None = None

    def __str__(self) -> str:
        return f"@{self.target}"


class AtAll(Element):
    def __str__(self) -> str:
        return "@全体成员"

    def __repr__(self) -> str:
        return "AtAll()"


class FingerGuessing(Element):
    @total_ordering
    class Choice(Enum):
        Rock = "石头"
        Scissors = "剪刀"
        Paper = "布"

        def __eq__(self, other: Self) -> bool:
            if not isinstance(other, FingerGuessing.Choice):
                raise TypeError(f"{other} 不是 FingerGuessing.Choice")
            return self.value == other.value

        def __lt__(self, other: Self) -> bool:
            if not isinstance(other, FingerGuessing.Choice):
                raise TypeError(f"{other} 不是 FingerGuessing.Choice")
            return (self.name, other.name) in {
                ("Rock", "Scissors"),
                ("Scissors", "Paper"),
                ("Paper", "Rock"),
            }

    choice: Choice

    def __init__(
        self,
        choice: Literal["Rock", "Paper", "Scissors" "石头", "剪刀", "布"] | Choice,
    ) -> None:
        C = FingerGuessing.Choice
        if isinstance(choice, str) and choice in ("Rock", "Paper", "Scissors"):
            self.choice = C[choice]
        if isinstance(choice, C):
            self.choice = choice
        raise TypeError(f"无效的猜拳参数：{choice}")

    def __str__(self) -> str:
        return f"[猜拳: {self.choice.value}]"

    def __repr__(self) -> str:
        return f"FingerGuessing(choice={self.choice})"


class Dice(Element):
    value: Literal[1, 2, 3, 4, 5, 6]

    def __init__(self, value: Literal[1, 2, 3, 4, 5, 6]) -> None:
        if value not in range(1, 6 + 1):
            raise ValueError(f"{value} 不是有效的骰子值")
        self.value = value

    def __str__(self) -> str:
        return f"[骰子: {self.value}]"

    def __repr__(self) -> str:
        return f"Dice(value={self.value})"
