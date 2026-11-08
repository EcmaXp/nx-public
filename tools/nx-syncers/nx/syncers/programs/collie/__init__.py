import os
import subprocess
from pathlib import Path

from ...types import Syncer


def collie_root() -> Path:
    return Path(os.environ.get("COLLIE_DIR") or Path.home() / ".local/share/collie")


def collie_binary() -> Path | None:
    root = collie_root()
    for candidate in (root / "current/bin/collie", root / "bin/collie"):
        if candidate.is_file():
            return candidate.resolve()
    return None


def sync_collie() -> bool:
    """Publish the standalone Collie's CLI through its native link command."""
    binary = collie_binary()
    if binary is None:
        return False
    link = Path.home() / ".local/bin/collie"
    if link.is_symlink() and link.resolve() == binary:
        return False
    subprocess.check_call([str(binary), "link"])
    return True


collie_syncer = Syncer(
    name="collie",
    sync=sync_collie,
    config_files=[],
    is_always_sync_required=True,
)
