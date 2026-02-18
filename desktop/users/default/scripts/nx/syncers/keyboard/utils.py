from __future__ import annotations

from collections.abc import Iterable

from .keymap import MODIFIERS

__all__ = ["sorted_modifiers"]


def sorted_modifiers(modifiers: Iterable[str]) -> tuple[str, ...]:
    return tuple(
        sorted(
            modifiers,
            key=lambda x: MODIFIERS.index(x) if x in MODIFIERS else len(MODIFIERS),
        ),
    )
