from __future__ import annotations

import enum
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, ClassVar, Self

from .keymap import MODIFIERS_ANY_MAP, MODIFIERS_HYPER_MAP, MODIFIERS_LEFT_TO_RIGHT_MAP
from .mapper import ModifierMapper
from .utils import sorted_modifiers

__all__ = [
    "Token",
    "Operation",
    "Assign",
    "Equal",
    "NotEqual",
    "MutableShortcut",
    "Shortcut",
]


class Token(enum.Enum):
    WHITESPACE = " +"
    IDENTIFIER = "@?[a-zA-Z0-9_][a-zA-Z0-9_. ]*[a-zA-Z0-9_]|@?[a-zA-Z0-9_]"
    STRING = "|".join([r'"[^"]*"', r"'[^']*'"])
    COMMA = ","
    STAR = r"\*"
    DASH = "-"
    CHAIN = ">"
    EQ = "=="
    NE = "!="
    ASSIGN = "="

    def __str__(self):
        return self.name


@dataclass(frozen=True)
class Operation:
    key: str
    value: str
    op = None

    if TYPE_CHECKING:
        op: ClassVar[Token]

    def __str__(self):
        return f"{self.key} {self.op.value} {self.value}"


@dataclass(frozen=True)
class Assign(Operation):
    op = Token.ASSIGN


@dataclass(frozen=True)
class Equal(Operation):
    op = Token.EQ


@dataclass(frozen=True)
class NotEqual(Operation):
    op = Token.NE


@dataclass
class MutableShortcut:
    optional_modifiers: list[str] = field(default_factory=list)
    mandatory_modifiers: list[str] = field(default_factory=list)
    key: str | None = None

    def freeze(self) -> Shortcut:
        if self.key is None:
            raise ValueError("Key is not set")

        return Shortcut(
            optional_modifiers=tuple(self.optional_modifiers),
            mandatory_modifiers=tuple(self.mandatory_modifiers),
            key=self.key,
        )


@dataclass(frozen=True)
class Shortcut:
    optional_modifiers: tuple[str, ...]
    mandatory_modifiers: tuple[str, ...]
    key: str

    def __gt__(self, other: Shortcut):
        if isinstance(other, Shortcut):
            return str(self) > str(other)

        return NotImplemented

    def __lt__(self, other: Shortcut):
        if isinstance(other, Shortcut):
            return str(self) < str(other)

        return NotImplemented

    def __str__(self):
        optional_modifiers = (
            ("*".join(sorted_modifiers(self.optional_modifiers)) + "*")
            if self.optional_modifiers
            else ""
        )
        mandatory_modifiers = (
            ("-".join(sorted_modifiers(self.mandatory_modifiers)) + "-")
            if self.mandatory_modifiers
            else ""
        )
        return f"{optional_modifiers}{mandatory_modifiers}{self.key}"

    def _convert_modifiers(self, func) -> Self:
        return type(self)(
            optional_modifiers=func(self.optional_modifiers),
            mandatory_modifiers=func(self.mandatory_modifiers),
            key=self.key,
        )

    def normalize(self) -> Self:
        return self._convert_modifiers(ModifierMapper(MODIFIERS_HYPER_MAP).forward)

    def alias(self) -> Self:
        return self._convert_modifiers(ModifierMapper(MODIFIERS_HYPER_MAP).backward)

    def to_left(self) -> Self:
        return self._convert_modifiers(
            ModifierMapper(MODIFIERS_LEFT_TO_RIGHT_MAP).backward
        )

    def to_right(self) -> Self:
        return self._convert_modifiers(
            ModifierMapper(MODIFIERS_LEFT_TO_RIGHT_MAP).forward
        )

    def to_any(self) -> Self:
        return self._convert_modifiers(ModifierMapper(MODIFIERS_ANY_MAP).forward)
