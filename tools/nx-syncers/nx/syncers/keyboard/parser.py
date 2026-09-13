from __future__ import annotations

import enum
import re

from .types import Assign, Equal, MutableShortcut, NotEqual, Shortcut, Token

__all__ = ["parse_shortcuts"]


class Lexer:
    def __init__(self, enum_cls: type[enum.Enum]):
        self.enum_cls = enum_cls
        self.pattern = re.compile(
            "|".join(
                f"(?P<{key}>{re.compile(value.value).pattern})"
                for key, value in enum_cls.__members__.items()
            )
        )

    def lex(self, text):
        pos = 0
        while pos < len(text):
            m = self.pattern.match(text, pos)
            if not m:
                raise ValueError(
                    f"no match: {text[max(pos - 10, 0) : pos]!r}, {text[pos : pos + 10]!r}"
                )

            yield self.enum_cls.__members__[m.lastgroup], m[0]
            pos = m.end()


def lex(text):
    return list(Lexer(Token).lex(text))


def split_tokens(all_tokens):
    tokens = [[]]

    for token, text in all_tokens:
        match token:
            case Token.COMMA:
                tokens.append([])
            case Token.WHITESPACE:
                pass
            case Token.STRING:
                tokens[-1].append((Token.IDENTIFIER, text[+1:-1]))
            case _:
                tokens[-1].append((token, text))

    return tokens


def build_operation(tokens):
    match tokens:
        case (Token.IDENTIFIER, key), (Token.EQ, _), (Token.IDENTIFIER, value):
            return Equal(key, value)
        case (Token.IDENTIFIER, key), (Token.NE, _), (Token.IDENTIFIER, value):
            return NotEqual(key, value)
        case (Token.IDENTIFIER, key), (Token.ASSIGN, _), (Token.IDENTIFIER, value):
            return Assign(key, value)
        case _:
            return build_shortcut(tokens)


def build_shortcut(tokens) -> list[Shortcut]:
    shortcuts = [MutableShortcut()]

    while tokens:
        match tokens[:2]:
            case (Token.IDENTIFIER, text), (Token.STAR, _):
                shortcuts[-1].optional_modifiers.append(text)
                del tokens[:2]
            case (Token.IDENTIFIER, text), (Token.DASH, _):
                shortcuts[-1].mandatory_modifiers.append(text)
                del tokens[:2]
            case (Token.IDENTIFIER, text), *_:
                assert shortcuts[-1].key is None
                shortcuts[-1].key = text
                del tokens[:1]
            case (Token.CHAIN, _), *_:
                shortcuts.append(MutableShortcut())
                del tokens[:1]
            case (Token.WHITESPACE, _), *_:
                del tokens[:1]
            case _:
                text = "".join(map(str, tokens))
                raise ValueError(f"unexpected tokens: {text}")

    return [shortcut.freeze() for shortcut in shortcuts]


def parse_shortcuts(text: str) -> list[list[Shortcut] | Assign | Equal]:
    all_tokens = lex(text)
    operations = []
    for tokens in split_tokens(all_tokens):
        operation = build_operation(tokens)
        operations.append(operation)

    return operations
