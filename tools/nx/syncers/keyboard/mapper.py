from __future__ import annotations

from .utils import sorted_modifiers

__all__ = ["KeyMapper", "ModifierMapper"]


class KeyMapper:
    def __init__(self, key_map: dict[str, str]):
        self.key_map = key_map
        self._forward_map = key_map
        self._backward_map = {v: k for k, v in key_map.items()}

    def forward(self, key: str) -> str:
        return self._forward_map.get(key, key)

    def backward(self, key: str) -> str:
        return self._backward_map.get(key, key)


class ModifierMapper:
    def __init__(self, modifiers_map: dict[str, str]):
        self.modifiers_map = modifiers_map
        self._forward_map = self._build_mapping(modifiers_map)
        self._backward_map = self._build_mapping(
            {v: k for k, v in modifiers_map.items()}
        )

    @staticmethod
    def _build_mapping(mapping: dict[str, str]):
        return {
            frozenset(k.split("-")): frozenset(v.split("-")) for k, v in mapping.items()
        }

    @classmethod
    def _convert_modifiers(
        cls,
        modifiers: tuple[str, ...],
        convert_map: dict[frozenset[str], frozenset[str]],
    ) -> tuple[str, ...]:
        modifiers = frozenset(modifiers)
        modifiers = convert_map.get(modifiers, modifiers)
        if modifiers:
            for key, value in convert_map.items():
                if key.issubset(modifiers):
                    modifiers = (modifiers - key) | value

        return sorted_modifiers(modifiers)

    def forward(self, modifiers: tuple[str, ...]) -> tuple[str, ...]:
        return self._convert_modifiers(modifiers, self._forward_map)

    def backward(self, modifiers: tuple[str, ...]) -> tuple[str, ...]:
        return self._convert_modifiers(modifiers, self._backward_map)
