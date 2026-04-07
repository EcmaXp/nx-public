from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path


@dataclass
class Syncer:
    name: str
    sync: Callable[[], bool]
    config_files: list[Path]
    is_always_sync_required: bool = False

    def resolve_config_files(self) -> list[Path]:
        return [file.resolve() for file in self.config_files]
