#!/usr/bin/env python3
import json
import shutil
from pathlib import Path
from typing import Any

from ...config import (
    KARABINER_JSON_GIT_PATH,
    load_keyboard_shortcuts,
    load_programs,
)
from ...keyboard.parser import parse_shortcuts
from ...keyboard.types import Assign, Equal, NotEqual, Shortcut
from .consts import ALLOWED_PROTOCOLS
from .keymap import MODIFIERS, PACKED_KEYS, convert_key, normalize_shortcut

KARABINER_LOCAL_CONFIG_PATH = Path("~/.config/karabiner/karabiner.json").expanduser()

PROGRAMS = {
    "aerospace": shutil.which("aerospace"),
    "ghostty-exec": shutil.which("ghostty-exec"),
    "killall": shutil.which("killall"),
    "open": shutil.which("open"),
    "zed": shutil.which("zed"),
}


def to_bool(value: str) -> bool:
    return {"true": True, "false": False}[value]


def to_typed(value: str) -> int | bool | str:
    try:
        return int(value)
    except ValueError:
        try:
            return to_bool(value)
        except KeyError:
            return value


def normalize_key(key: str):
    operations = []
    for operation in parse_shortcuts(key):
        if isinstance(operation, list):
            operations.append(">".join(map(str, operation)))
        else:
            operations.append(str(operation))

    return ", ".join(operations)


def parse_applications(text: str) -> tuple[str, list[str]]:
    applications = []
    descriptions = list(filter(None, map(str.strip, text.split(","))))

    programs = load_programs()
    for item in descriptions[:]:
        if item in programs:
            applications.append(item)
            descriptions.remove(item)

    text = ", ".join(descriptions)
    return text, applications


def parse_conditions(text: str) -> tuple[str, list[Equal | NotEqual]]:
    operations = []
    descriptions = list(filter(None, map(str.strip, text.split(","))))

    for item in descriptions[:]:
        for operation in parse_shortcuts(item):
            if isinstance(operation, Equal | NotEqual):
                operations.append(operation)
                descriptions.remove(item)

    return ", ".join(descriptions), operations


def preprocess_special_shortcuts(shortcuts: dict):
    new_shortcuts = {}
    chain_conds = set()
    non_modifier_conds = set()
    non_modifier_alone = {}

    for description, keymap in shortcuts.items():
        new_shortcuts[description] = new_keymap = {}
        for key, value in keymap.items():
            key, key_applications = parse_applications(key)
            key = normalize_key(key)

            if key in non_modifier_alone:
                non_modifier_alone[key] = value
                continue

            for operation in parse_shortcuts(key):
                key, key_chain_conds = get_chain_shortcuts(key, operation)
                chain_conds.update(key_chain_conds)

                key, key_non_modifier_conds = get_non_modifier_shortcuts(key, operation)
                non_modifier_conds.update(key_non_modifier_conds)
                for key_non_modifier_cond in key_non_modifier_conds:
                    non_modifier_alone.setdefault(str(key_non_modifier_cond))

                if key_chain_conds and key_non_modifier_conds:
                    raise ValueError("both chain and non-modifier shortcuts")

            new_keymap[", ".join([key, *key_applications])] = value

    if chain_conds or non_modifier_conds:
        new_keymap = new_shortcuts.setdefault("Conditional Keys", {})

        for chain_cond in sorted(chain_conds):
            new_keymap[str(chain_cond)] = {
                "parameters": {
                    "basic.to_if_held_down_threshold_milliseconds": 0,
                },
                "to_if_held_down": f"{str(chain_cond)!r} = true, modified = true",
                "to_delayed_action": {
                    "to_if_invoked": f"{str(chain_cond)!r} = false, modified = false",
                    "to_if_canceled": f"{str(chain_cond)!r} = false, modified = false",
                },
            }

        for non_modifier_cond in sorted(non_modifier_conds):
            new_keymap[str(non_modifier_cond)] = {
                "parameters": {
                    "basic.to_if_held_down_threshold_milliseconds": 0,
                },
                "to_if_alone": (
                    non_modifier_alone.get(str(non_modifier_cond))
                    or str(non_modifier_cond)
                ),
                "to_if_held_down": f"{str(non_modifier_cond)!r} = true, modified = true",
                "to_after_key_up": f"{str(non_modifier_cond)!r} = false, modified = false",
            }

        for modifier in [
            "rshift",
        ]:
            new_keymap[f"{modifier}-@any key_code"] = "none"

        for modifier in [
            "hyper",
            "hyper-cmd",
            "hyper-alt",
            "hyper-shift",
            "hyper-ctrl",
        ]:
            new_keymap[f"{modifier}-@any key_code, modified == true"] = "none"

    return new_shortcuts


def get_chain_shortcuts(key: str, operation) -> tuple[str, list[Shortcut]]:
    condition_shortcuts = []
    if isinstance(operation, list) and is_chain_shortcuts(operation):
        condition_shortcut, rest_shortcut = parse_chain_shortcut(operation)
        condition_shortcuts.append(condition_shortcut)
        key = key.replace(
            ">".join(map(str, operation)),
            f"{str(condition_shortcut)!r} == true, {rest_shortcut}",
        )
        operation = [rest_shortcut]

    return key, condition_shortcuts


def get_non_modifier_shortcuts(key: str, operation) -> tuple[str, list[Shortcut]]:
    condition_shortcuts = []
    if isinstance(operation, list):
        for shortcut in operation:
            if is_non_modifier_shortcut(shortcut):
                condition_shortcut, rest_shortcut = parse_non_modifier_shortcut(
                    shortcut
                )
                condition_shortcuts.append(condition_shortcut)
                key = key.replace(
                    str(shortcut),
                    f"{str(condition_shortcut)!r} == true, {rest_shortcut}",
                )

    return key, condition_shortcuts


def is_chain_shortcuts(operation: list) -> bool:
    match len(operation):
        case 0 | 1:
            return False
        case 2:
            return True
        case _:
            raise ValueError("too many shortcuts in chain")


def is_non_modifier_shortcut(shortcut: Shortcut) -> bool:
    for modifier in shortcut.optional_modifiers:
        if modifier not in MODIFIERS:
            raise ValueError("non-modifier in optional modifiers")

    return any(modifier not in MODIFIERS for modifier in shortcut.mandatory_modifiers)


def parse_chain_shortcut(operation: list) -> tuple[Shortcut, Shortcut]:
    assert is_chain_shortcuts(operation)
    condition_shortcut, rest_shortcut = operation
    return condition_shortcut, rest_shortcut


def parse_non_modifier_shortcut(shortcut: Shortcut) -> tuple[Shortcut, Shortcut]:
    assert is_non_modifier_shortcut(shortcut)
    assert shortcut.key is not None

    condition_shortcut = Shortcut(
        optional_modifiers=(),
        mandatory_modifiers=shortcut.mandatory_modifiers[:-1],
        key=shortcut.mandatory_modifiers[-1],
    )

    rest_shortcut = Shortcut(
        optional_modifiers=(),
        mandatory_modifiers=tuple(
            modifier
            for modifier in shortcut.mandatory_modifiers
            if modifier in MODIFIERS
        ),
        key=shortcut.key,
    )

    return condition_shortcut, rest_shortcut


def build_rules(shortcuts: dict):
    rules = [
        build_rule(description, keymap) for description, keymap in shortcuts.items()
    ]
    return rules


def build_rule(description: str, keymap: dict[str, str | dict]):
    description, rule_applications = parse_applications(description)
    description, rule_conditions = parse_conditions(description)
    if rule_applications:
        if description:
            description = f"[{', '.join(rule_applications)}] {description}"
        else:
            description = f"[{', '.join(rule_applications)}]"

    return {
        "description": description,
        "manipulators": build_manipulators(keymap, rule_applications, rule_conditions),
    }


def replace_obj[T: dict | str](obj: T, old: str, new: str) -> T | Any:
    if isinstance(obj, dict):
        return {
            replace_obj(key, old, new): replace_obj(value, old, new)
            for key, value in obj.items()
        }
    elif isinstance(obj, str):
        return obj.replace(old, new)
    elif isinstance(obj, list):
        return [replace_obj(item, old, new) for item in obj]
    else:
        raise TypeError(f"unexpected type: {type(obj)}")


def build_manipulators(
    keymap: dict,
    applications: list[str],
    conditions: list[Equal | NotEqual],
):
    manipulators = []
    for key, value in keymap.items():
        packed_keys = {}
        for item in (key, value):
            for old_token, new_tokens in PACKED_KEYS.items():
                if old_token in item:
                    packed_keys[old_token] = new_tokens

        if packed_keys:
            for token, new_tokens in packed_keys.items():
                for new_token in new_tokens:
                    new_key = replace_obj(key, token, new_token)
                    new_value = replace_obj(value, token, new_token)
                    manipulators.append(
                        build_manipulator(new_key, new_value, applications, conditions)
                    )
        else:
            manipulators.append(build_manipulator(key, value, applications, conditions))

    return manipulators


def build_manipulator(
    key: str,
    value: str | dict,
    applications: list[str],
    conditions: list[Equal | NotEqual],
):
    key, key_applications = parse_applications(key)
    manipulator_applications = applications + key_applications

    manipulator = (
        {"type": "basic"}
        | build_manipulator_conditions(key, manipulator_applications, conditions)
        | build_manipulator_from(key)
        | build_manipulator_to(value)
    )
    return manipulator


def build_manipulator_conditions(
    condition: str,
    applications: list[str],
    conditions: list[Equal | NotEqual],
):
    manipulator = {"conditions": []}

    for operation in conditions + parse_shortcuts(condition):
        if isinstance(operation, Equal | NotEqual):
            suffix = {
                Equal: "if",
                NotEqual: "unless",
            }[type(operation)]

            if operation.key == "@language":
                manipulator["conditions"].append(
                    {
                        "type": f"input_source_{suffix}",
                        "input_sources": [{"language": operation.value}],
                    }
                )
            elif operation.key.startswith("@"):
                raise ValueError(operation)
            else:
                manipulator["conditions"].append(
                    {
                        "type": f"variable_{suffix}",
                        "name": operation.key,
                        "value": to_typed(operation.value),
                    }
                )
        elif isinstance(operation, list):
            pass  # processed at build_manipulator_from
        else:
            raise ValueError(operation)

    if applications:
        programs = load_programs()
        bundle_identifiers = []
        for application in applications:
            if application in programs:
                bundle_identifier = programs[application]
                if isinstance(bundle_identifier, str):
                    bundle_identifiers.append(bundle_identifier)
                else:
                    bundle_identifiers.extend(bundle_identifier)

        manipulator["conditions"].append(
            {
                "type": "frontmost_application_if",
                "description": ", ".join(applications),
                "bundle_identifiers": bundle_identifiers,
            }
        )

    return manipulator if manipulator["conditions"] else {}


def build_manipulator_from(condition: str):
    manipulator = {"from": {}}
    for operation in parse_shortcuts(condition):
        if isinstance(operation, list):
            for shortcut in operation:
                shortcut = normalize_shortcut(shortcut)

                manipulator["from"].update(convert_key(shortcut.key))

                for modifier_key, modifiers in {
                    "mandatory": shortcut.mandatory_modifiers,
                    "optional": shortcut.optional_modifiers,
                }.items():
                    if modifiers:
                        manipulator["from"].setdefault("modifiers", {})
                        manipulator["from"]["modifiers"][modifier_key] = list(modifiers)
        elif isinstance(operation, Equal | NotEqual):
            pass  # processed at build_manipulator_conditions
        else:
            raise ValueError(operation)

    return manipulator


def build_manipulator_to(operations: str | dict):
    if isinstance(operations, dict):
        manipulator = {}
        for to_key in "to", "to_if_alone", "to_if_held_down", "to_after_key_up":
            if to_key in operations:
                manipulator[to_key] = _build_manipulator_to_keys(operations[to_key])

        if "to_delayed_action" in operations:
            manipulator["to_delayed_action"] = {}
            for to_key in "to_if_invoked", "to_if_canceled":
                if to_key in operations["to_delayed_action"]:
                    manipulator["to_delayed_action"][to_key] = (
                        _build_manipulator_to_keys(
                            operations["to_delayed_action"][to_key]
                        )
                    )

        if "parameters" in operations:
            manipulator["parameters"] = operations["parameters"]

        missing_keys = operations.keys() - {
            "to",
            "to_if_alone",
            "to_if_held_down",
            "to_after_key_up",
            "to_delayed_action",
            "parameters",
        }
        if missing_keys:
            raise ValueError(missing_keys)

        return manipulator
    else:
        return {"to": _build_manipulator_to_keys(operations)}


def _build_manipulator_to_keys(operations: str | dict) -> list[dict]:
    program_keys = _build_manipulator_to_keys_with_programs(operations)
    if program_keys:
        return program_keys

    keys = []
    if isinstance(operations, dict):
        return [operations]
    elif isinstance(operations, list):
        for operations in operations:
            keys += _build_manipulator_to_keys(operations)
        return keys

    for operation in parse_shortcuts(operations):
        if isinstance(operation, list):
            for shortcut in operation:
                shortcut = normalize_shortcut(shortcut)
                keys.append(convert_key(shortcut.key))
                if shortcut.mandatory_modifiers:
                    keys[-1]["modifiers"] = list(shortcut.mandatory_modifiers)
                if shortcut.optional_modifiers:
                    raise ValueError("optional modifiers are not supported")
        elif isinstance(operation, Assign):
            if operation.key in ("@halt", "@repeat"):
                key = operation.key.removeprefix("@")
                value = to_typed(operation.value)
                keys[-1][key] = value
            elif operation.key.startswith("@"):
                raise ValueError(operation)
            else:
                key = operation.key
                value = to_typed(operation.value)
                keys.append({"set_variable": {"name": key, "value": value}})
        else:
            raise ValueError(operation)

    return keys


def _build_manipulator_to_keys_with_programs(operations: str | dict) -> dict | None:
    if not isinstance(operations, str):
        return None

    programs = load_programs()

    if operations in programs:
        return {"shell_command": f"open -b {programs[operations]}"}

    if "##" in operations:
        operations, program = operations.split("##")
        assert '"' not in operations

        if operations.startswith("~/"):
            operations = "$HOME/" + operations.removeprefix("~/")

        if program:
            bundle_id = programs[program]
            return {"shell_command": f'open -b {bundle_id} "{operations}"'}
        else:
            return {"shell_command": f'open "{operations}"'}

    if operations.startswith(ALLOWED_PROTOCOLS):
        assert '"' not in operations
        return {"shell_command": f'open "{operations}"'}

    for program, location in PROGRAMS.items():
        if operations.startswith(f"{program} ") and "=" not in operations:
            if location is None:
                raise ValueError(f"location is not defined for {program}")

            return {"shell_command": operations.replace(program, location)}

    if operations.startswith("shell "):
        return {"shell_command": operations.removeprefix("shell ").lstrip()}

    return None


def save_rules(rules) -> bool:
    with KARABINER_LOCAL_CONFIG_PATH.open("r") as f:
        config = json.load(f)

    updated = False
    if config["profiles"][0]["complex_modifications"]["rules"] != rules:
        config["profiles"][0]["complex_modifications"]["rules"] = rules
        updated = True

    with KARABINER_LOCAL_CONFIG_PATH.open("w") as f:
        json.dump(config, f, indent=2, sort_keys=False)

    return updated


def save_rules_to_git():
    with KARABINER_LOCAL_CONFIG_PATH.open("r") as f:
        config = json.load(f)

    with KARABINER_JSON_GIT_PATH.open("w") as f:
        json.dump(config, f, indent=2, sort_keys=True)


def sync_karabiner() -> bool:
    load_programs.cache_clear()
    load_keyboard_shortcuts.cache_clear()

    shortcuts = load_keyboard_shortcuts()
    shortcuts = preprocess_special_shortcuts(shortcuts)
    rules = build_rules(shortcuts)
    updated = save_rules(rules)
    save_rules_to_git()
    return updated
