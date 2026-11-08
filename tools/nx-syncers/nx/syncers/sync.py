import dbm
import sys
import traceback
from datetime import datetime
from pathlib import Path

import click
from watchdog.events import PatternMatchingEventHandler
from watchdog.observers import Observer

from .config import CONFIG_ALL_FOLDERS, SYNCERS_USER_DB_PATH
from .programs.aerospace import aerospace_syncer
from .programs.collie import collie_syncer
from .programs.karabiner import karabiner_syncer
from .types import Syncer
from .utils import now

SYNCERS = {
    syncer.name: syncer
    for syncer in [
        aerospace_syncer,
        collie_syncer,
        karabiner_syncer,
    ]
}


class SyncFailed(click.ClickException):
    def __init__(self, names: list[str]):
        super().__init__(f"{', '.join(names)} failed to sync")


def modified_at_key(file: Path) -> str:
    return f"last_modified_at:{file.as_posix()}"


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
    modified: dict[Path, bytes] = {}
    unsynced: set[Path] = set()
    failed: list[str] = []
    for syncer in syncers.values():
        sync_required = syncer.is_always_sync_required or force
        owned = [file for file in files if file in syncer.resolve_config_files()]
        for file in owned:
            modified_at = datetime.fromtimestamp(file.stat().st_mtime)
            value = modified_at.isoformat(timespec="seconds").encode("utf-8")
            if db.get(modified_at_key(file)) != value:
                sync_required = True
                if file not in modified:
                    print(f"[{now(modified_at)}] {file.name} modified")
                modified[file] = value

        if sync_required:
            try:
                synced = syncer.sync()
            except Exception:
                traceback.print_exc()
                print(f"[{now()}] {syncer.name} sync failed", file=sys.stderr)
                failed.append(syncer.name)
                unsynced.update(owned)
                continue
            if synced:
                print(f"[{now()}] {syncer.name} synced")

    for file, value in modified.items():
        if file not in unsynced:
            db[modified_at_key(file)] = value
    if failed:
        raise SyncFailed(failed)


class SyncHandler(PatternMatchingEventHandler):
    def __init__(
        self,
        db_path: Path,
        syncers: dict[str, Syncer],
        force: bool = False,
    ):
        super().__init__(patterns=["*.yaml"])
        self.db_path = db_path
        self.syncers = syncers
        self.force = force

    def on_modified(self, event):
        path = Path(event.src_path).resolve()
        try:
            with dbm.open(self.db_path, "c") as db:
                sync(db=db, syncers=self.syncers, force=self.force, files=[path])
        except SyncFailed:
            print(f"[{now()}] sync failed, still watching", file=sys.stderr)
        except Exception:
            traceback.print_exc()
            print(f"[{now()}] sync failed, still watching", file=sys.stderr)


def sync_forever(
    db,
    syncers: dict[str, Syncer],
    force: bool = False,
    db_path: Path = SYNCERS_USER_DB_PATH,
):
    sync(db=db, syncers=syncers, force=force)
    print(f"[{now()}] watching for changes...")

    event_handler = SyncHandler(db_path=db_path, syncers=syncers, force=force)

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
