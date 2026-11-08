import dbm
import io
import tempfile
import threading
import unittest
from contextlib import redirect_stderr
from pathlib import Path

from watchdog.events import FileModifiedEvent

from . import sync as sync_module
from .types import Syncer


class SyncHandlerTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.dir = Path(tmp.name)
        self.config = self.dir / "a.yaml"
        self.config.write_text("a: 1\n")
        self.db_path = self.dir / "db"

    def make_handler(self, syncer):
        return sync_module.SyncHandler(db_path=self.db_path, syncers={"t": syncer})

    def dispatch_in_thread(self, handler):
        errors = []

        def run():
            try:
                handler.on_modified(FileModifiedEvent(str(self.config)))
            except BaseException as e:
                errors.append(e)

        thread = threading.Thread(target=run)
        thread.start()
        thread.join(10)
        return errors

    def test_handler_syncs_from_a_watchdog_thread(self):
        calls = []
        syncer = Syncer(
            name="t",
            sync=lambda: calls.append(1) or True,
            config_files=[self.config],
        )
        handler = self.make_handler(syncer)
        with dbm.open(self.db_path, "c"):
            errors = self.dispatch_in_thread(handler)
        self.assertEqual(errors, [])
        self.assertEqual(calls, [1])

    def test_handler_survives_a_failing_syncer(self):
        def boom():
            raise RuntimeError("boom")

        handler = self.make_handler(
            Syncer(name="t", sync=boom, config_files=[self.config])
        )
        with redirect_stderr(io.StringIO()) as err:
            errors = self.dispatch_in_thread(handler)
        self.assertEqual(errors, [])
        self.assertIn("boom", err.getvalue())

    def test_a_failing_syncer_prints_one_traceback_per_event(self):
        def boom():
            raise RuntimeError("boom")

        handler = self.make_handler(
            Syncer(name="t", sync=boom, config_files=[self.config])
        )
        with redirect_stderr(io.StringIO()) as err:
            self.dispatch_in_thread(handler)
        self.assertEqual(err.getvalue().count("Traceback"), 1)
        self.assertIn("sync failed, still watching", err.getvalue())


class SyncTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.dir = Path(tmp.name)
        self.db = {}

    def config(self, name):
        file = self.dir / name
        file.write_text("a: 1\n")
        return file.resolve()

    def run_sync(self, syncers):
        with redirect_stderr(io.StringIO()):
            try:
                sync_module.sync(db=self.db, syncers=syncers)
            except sync_module.SyncFailed as e:
                return e
        return None

    def test_a_failing_syncer_does_not_stop_the_syncers_after_it(self):
        calls = []

        def boom():
            raise RuntimeError("boom")

        syncers = {
            "a": Syncer(name="a", sync=boom, config_files=[self.config("a.yaml")]),
            "b": Syncer(
                name="b",
                sync=lambda: calls.append("b") or True,
                config_files=[self.config("b.yaml")],
            ),
        }
        error = self.run_sync(syncers)
        self.assertEqual(calls, ["b"])
        self.assertIsNotNone(error)
        self.assertEqual(error.message, "a failed to sync")
        self.assertEqual(error.exit_code, 1)

    def test_a_failed_sync_leaves_its_file_unrecorded_so_the_next_run_retries(self):
        attempts = []

        def flaky():
            attempts.append(1)
            if len(attempts) == 1:
                raise RuntimeError("busy")
            return True

        syncers = {
            "a": Syncer(name="a", sync=flaky, config_files=[self.config("a.yaml")])
        }
        self.assertIsNotNone(self.run_sync(syncers))
        self.assertIsNone(self.run_sync(syncers))
        self.assertEqual(len(attempts), 2)
        self.assertIsNone(self.run_sync(syncers))
        self.assertEqual(len(attempts), 2)

    def test_a_shared_file_stays_unrecorded_when_one_of_its_syncers_fails(self):
        shared = self.config("programs.yaml")
        attempts = []

        def flaky():
            attempts.append(1)
            if len(attempts) == 1:
                raise RuntimeError("busy")
            return True

        syncers = {
            "karabiner": Syncer(name="karabiner", sync=flaky, config_files=[shared]),
            "raycast": Syncer(
                name="raycast",
                sync=lambda: True,
                config_files=[shared],
                is_always_sync_required=True,
            ),
        }
        self.run_sync(syncers)
        self.run_sync(syncers)
        self.assertEqual(len(attempts), 2)


if __name__ == "__main__":
    unittest.main()
