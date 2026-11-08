import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from . import collie_binary, sync_collie


class CollieTests(unittest.TestCase):
    def test_missing_install_skips_instead_of_raising(self):
        with (
            tempfile.TemporaryDirectory() as root,
            patch.dict(os.environ, {"COLLIE_DIR": root}),
            patch("subprocess.check_call") as call,
        ):
            self.assertFalse(sync_collie())
        call.assert_not_called()

    def test_current_version_wins_over_the_clone_root(self):
        with (
            tempfile.TemporaryDirectory() as root,
            patch.dict(os.environ, {"COLLIE_DIR": root}),
        ):
            for rel in ("bin/collie", "versions/v1/bin/collie"):
                path = Path(root) / rel
                path.parent.mkdir(parents=True)
                path.write_text("")
            (Path(root) / "current").symlink_to(Path(root) / "versions/v1")
            self.assertEqual(
                collie_binary(), (Path(root) / "versions/v1/bin/collie").resolve()
            )


if __name__ == "__main__":
    unittest.main()
