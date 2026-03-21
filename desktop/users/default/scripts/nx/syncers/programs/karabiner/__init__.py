from ...config import (
    KEYBOARD_CONFIG_PATH,
    KEYBOARD_USER_CONFIG_PATH,
    PROGRAMS_CONFIG_PATH,
)
from ...types import Syncer
from .sync import sync_karabiner

karabiner_syncer = Syncer(
    name="karabiner",
    sync=sync_karabiner,
    config_files=[
        KEYBOARD_CONFIG_PATH,
        KEYBOARD_USER_CONFIG_PATH,
        PROGRAMS_CONFIG_PATH,
    ],
)
