import json
import shutil
import subprocess
from pathlib import Path

from ...types import Syncer


def sync_collie() -> bool:
    """Publish the installed Collie plugin's CLI through its native link command."""
    if shutil.which("herdr") is None:
        return False
    result = json.loads(
        subprocess.check_output(
            ["herdr", "plugin", "list", "--plugin", "herdr.collie", "--json"],
            text=True,
        )
    )
    plugins = result["result"]["plugins"]
    if not plugins:
        return False
    (plugin,) = plugins
    binary = Path(plugin["plugin_root"]) / "bin/collie"
    link = Path.home() / ".local/bin/collie"
    if link.is_symlink() and link.readlink() == binary:
        return False
    subprocess.check_call([str(binary), "link"])
    return True


collie_syncer = Syncer(
    name="collie",
    sync=sync_collie,
    config_files=[],
    is_always_sync_required=True,
)
