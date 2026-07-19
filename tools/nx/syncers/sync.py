import dbm
import sys
from datetime import datetime
from pathlib import Path

import click
from watchdog.events import PatternMatchingEventHandler
from watchdog.observers import Observer

from .config import CONFIG_ALL_FOLDERS, SYNCERS_USER_DB_PATH
from .programs.aerospace import aerospace_syncer
from .programs.karabiner import karabiner_syncer
from .types import Syncer
from .utils import now

SYNCERS = {
    syncer.name: syncer
    for syncer in [
        aerospace_syncer,
        karabiner_syncer,
    ]
}


def check_and_update_file_modified_at(
    db: dict,
    file: Path,
    modified_at: datetime,
) -> bool:
    key = f"last_modified_at:{file.as_posix()}"
    value = modified_at.isoformat(timespec="seconds").encode("utf-8")
    if db.get(key) != value:
        db[key] = value
        return True
    else:
        return False


def sync(
    db,
    syncers: dict[str, Syncer],
    force: bool = False,
    files: list[Path] | None = None,
):
    if files is not None:
        files = [file.resolve() for file in files]
    else:
        files = [
            file
            for syncer in syncers.values()
            for file in syncer.resolve_config_files()
        ]

    files = sorted(set(files), key=files.index)
    for syncer in syncers.values():
        sync_required = syncer.is_always_sync_required or force
        for file in files:
            if file in syncer.resolve_config_files():
                modified_at = datetime.fromtimestamp(file.stat().st_mtime)
                if check_and_update_file_modified_at(
                    db=db,
                    file=file,
                    modified_at=modified_at,
                ):
                    sync_required |= True
                    print(f"[{now(modified_at)}] {file.name} modified")

        if sync_required:
            synced = syncer.sync()
            if synced:
                print(f"[{now()}] {syncer.name} synced")


def sync_forever(
    db,
    syncers: dict[str, Syncer],
    force: bool = False,
):
    sync(db=db, syncers=syncers, force=force)
    print(f"[{now()}] watching for changes...")

    class SyncHandler(PatternMatchingEventHandler):
        def on_modified(self, event):
            path = Path(event.src_path).resolve()
            sync(db=db, syncers=syncers, force=force, files=[path])

    event_handler = SyncHandler(patterns=["*.yaml"])

    observer = Observer()
    for folder in {
        file.resolve().parent
        for folder in CONFIG_ALL_FOLDERS
        for file in folder.glob("*.yaml")
    } | set(CONFIG_ALL_FOLDERS):
        observer.schedule(event_handler, folder)

    observer.start()

    try:
        while observer.is_alive():
            observer.join(1)
    except KeyboardInterrupt:
        pass
    finally:
        observer.stop()
        observer.join()


@click.command()
@click.argument("programs", nargs=-1)
@click.option("--forever", is_flag=True)
@click.option("--force", is_flag=True)
def main(
    programs: tuple[str, ...] = (),
    forever: bool = False,
    force: bool = False,
):
    if programs:
        try:
            syncers = {program: SYNCERS[program] for program in programs}
        except KeyError as e:
            sys.exit(f"unknown program: {e}")
    else:
        syncers = SYNCERS

    with dbm.open(SYNCERS_USER_DB_PATH, "c") as db:
        if forever:
            sync_forever(db=db, syncers=syncers, force=force)
        else:
            sync(db=db, syncers=syncers, force=force)


if __name__ == "__main__":
    main()
