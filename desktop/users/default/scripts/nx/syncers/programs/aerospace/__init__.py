from ...config import AEROSPACE_CONFIG_PATH
from ...types import Syncer
from .sync import sync_aerospace

aerospace_syncer = Syncer(
    name="aerospace",
    sync=sync_aerospace,
    config_files=[
        AEROSPACE_CONFIG_PATH,
    ],
)
