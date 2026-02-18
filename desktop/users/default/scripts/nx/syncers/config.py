import os
from datetime import datetime
from functools import cache
from pathlib import Path
from time import sleep

import yaml

USER = os.environ["USER"]
NIX_REPO = next(
    parent for parent in Path(__file__).parents if (parent / ".git").exists()
)
NIX_DESKTOP = NIX_REPO / "desktop"
CONFIG_FOLDER = NIX_DESKTOP / "users"
SYNCERS_USER_CACHE_FOLDER = CONFIG_FOLDER / USER / ".cache"
SYNCERS_USER_DB_PATH = SYNCERS_USER_CACHE_FOLDER / f"syncers-{USER}"
CONFIG_USER_FOLDER = CONFIG_FOLDER / USER / f"nx-{USER}"
CONFIG_ALL_FOLDERS = (CONFIG_FOLDER, CONFIG_USER_FOLDER)

PROGRAMS_CONFIG_PATH = CONFIG_FOLDER / "programs.yaml"
KEYBOARD_CONFIG_PATH = CONFIG_FOLDER / "keyboard.yaml"
KEYBOARD_USER_CONFIG_PATH = CONFIG_USER_FOLDER / f"keyboard-{USER}.yaml"
KARABINER_JSON_GIT_PATH = CONFIG_USER_FOLDER / "karabiner.json"

AEROSPACE_FOLDER = NIX_DESKTOP / "users/default/programs/aerospace"
AEROSPACE_CONFIG_PATH = AEROSPACE_FOLDER / "aerospace.toml"


@(lambda func: yaml.add_representer(str, func, Dumper=yaml.SafeDumper) or func)
def represent_long_str(
    dumper,
    data,
    *,
    represent_str=yaml.SafeDumper.represent_str,
):
    if "\n" in data:
        return dumper.represent_scalar("tag:yaml.org,2002:str", data, style="|")

    return represent_str(dumper, data)


def load_config(file: Path) -> dict:
    with file.open(encoding="utf-8") as f:
        return yaml.safe_load(f)


def save_config(file: Path, config: dict):
    with file.open("w") as f:
        yaml.safe_dump(
            config,
            f,
            sort_keys=False,
            allow_unicode=True,
            width=1000,
        )


def watch_config(path: Path):
    last_modified_at = None
    while True:
        modified_at = path.stat().st_mtime
        if last_modified_at != modified_at:
            last_modified_at = modified_at

            print(
                f"{path.name} modified @",
                datetime.fromtimestamp(modified_at).isoformat(" ", "seconds"),
            )

            yield

        sleep(0.1)


@cache
def load_programs():
    return load_config(PROGRAMS_CONFIG_PATH)["programs"]


@cache
def load_keyboard_shortcuts():
    global_shortcuts = load_config(KEYBOARD_CONFIG_PATH)["keyboard"]["shortcuts"]
    user_shortcuts = load_config(KEYBOARD_USER_CONFIG_PATH)["keyboard"]["shortcuts"]
    return global_shortcuts | {f"{USER}, {k}": v for k, v in user_shortcuts.items()}


