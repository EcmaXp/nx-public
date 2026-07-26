MODIFIERS = (
    "any",
    "hyper",
    *("cmd", "ctrl", "alt", "shift"),
    *("lcmd", "lctrl", "lalt", "lshift"),
    *("rcmd", "rctrl", "ralt", "rshift"),
    "fn",
    "caps_lock",
)

MODIFIERS_HYPER_MAP = {
    "hyper": "rcmd-rctrl-ralt-rshift",
    "hyper-cmd": "rctrl-ralt-rshift",
    "hyper-ctrl": "rcmd-ralt-rshift",
    "hyper-alt": "rcmd-rctrl-rshift",
    "hyper-shift": "rcmd-rctrl-ralt",
}

MODIFIERS_LEFT_TO_RIGHT_MAP = {
    **{"lcmd": "rcmd", "lctrl": "rctrl", "lalt": "ralt", "lshift": "rshift"},
}

MODIFIERS_ANY_MAP = {
    **{"lcmd": "cmd", "lctrl": "ctrl", "lalt": "alt", "lshift": "shift"},
    **{"rcmd": "cmd", "rctrl": "ctrl", "ralt": "alt", "rshift": "shift"},
}
