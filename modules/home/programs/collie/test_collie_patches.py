#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = [
#     "brotli>=1.1",
#     "fonttools[woff]>=4.55",
#     "httpx>=0.27",
#     "pydantic>=2.9",
#     "typer>=0.12",
#     "toon-format>=1.1",
# ]
# ///
"""Run the collie_patches self-checks with uv run --script test_collie_patches.py."""

from __future__ import annotations

import io
import os
import stat
import subprocess
import tempfile
from collections.abc import Callable, Iterator
from contextlib import contextmanager, redirect_stdout
from pathlib import Path
from unittest.mock import patch

import collie_patches as cp

REGISTRY = "bridge/mux/registry.ts"
REGISTRY_V1 = 'a\nb\nconst MUX_ADAPTERS = ["tmux"];\nc\n'
REX_REGISTRY = 'a\nb\nconst MUX_ADAPTERS = ["tmux", "rex"];\nc\n'
MOVED_REGISTRY = 'a\nB\nconst MUX_ADAPTERS = ["tmux", "zellij"];\nc\n'
GIT_ENV = {
    "GIT_CONFIG_GLOBAL": os.devnull,
    "GIT_CONFIG_NOSYSTEM": "1",
    "GIT_AUTHOR_NAME": "t",
    "GIT_AUTHOR_EMAIL": "user@example.com",
    "GIT_COMMITTER_NAME": "t",
    "GIT_COMMITTER_EMAIL": "user@example.com",
}


def sh(cwd: Path, *argv: str) -> str:
    return subprocess.run(
        argv, cwd=cwd, check=True, capture_output=True, text=True
    ).stdout


def commit_release(
    src: Path, version: str, files: dict[str, str], annotated: bool = False
) -> None:
    for rel, text in files.items():
        (src / rel).parent.mkdir(parents=True, exist_ok=True)
        (src / rel).write_text(text)
    (src / "herdr-plugin.toml").write_text(f'version = "{version}"\n')
    sh(src, "git", "add", "-A")
    sh(src, "git", "commit", "-qm", version)
    if annotated:
        sh(src, "git", "tag", "-a", "-m", version, f"v{version}")
    else:
        sh(src, "git", "tag", f"v{version}")


@contextmanager
def collie(next_release: dict[str, str]) -> Iterator[Path]:
    """A checkout on v1.0.0 whose origin also has v1.0.1, with rex.patch recorded against v1.0.0."""
    with tempfile.TemporaryDirectory() as tmp, patch.dict(os.environ, GIT_ENV):
        root = Path(tmp)
        src = root / "src"
        src.mkdir()
        sh(src, "git", "init", "-q", "-b", "main")
        commit_release(src, "1.0.0", {REGISTRY: REGISTRY_V1})
        (src / REGISTRY).write_text(REX_REGISTRY)
        (src / cp.REX_MARKER).parent.mkdir(parents=True)
        (src / cp.REX_MARKER).write_text("export {};\n")
        sh(src, "git", "add", "-A")
        rex_patch = root / "rex.patch"
        rex_patch.write_text(sh(src, "git", "diff", "--cached", "HEAD"))
        sh(src, "git", "reset", "-q", "--hard", "v1.0.0")
        commit_release(src, "1.0.1", next_release)
        sh(root, "git", "clone", "-q", "--bare", str(src), "origin.git")
        checkout = root / "checkout"
        sh(root, "git", "clone", "-q", str(root / "origin.git"), str(checkout))
        sh(checkout, "git", "checkout", "-q", "--detach", "v1.0.0")
        rex_tests_patch = root / "rex-tests.patch"
        with (
            patch.object(cp, "checkout_dir", lambda: checkout),
            patch.object(cp, "BACKUP_DIR", root / "backup"),
            patch.object(cp, "REX_PATCH", rex_patch),
            patch.object(cp, "REX_TEST_PATCH", rex_tests_patch),
            patch.object(cp, "REX_PATCHES", (rex_patch, rex_tests_patch)),
        ):
            yield checkout


@contextmanager
def updater(on_run: Callable[[], None] = lambda: None) -> Iterator[list[list[str]]]:
    """Stand in for `bun run cli/main.ts update`, leaving every other subprocess real."""
    calls: list[list[str]] = []
    real = subprocess.run

    def fake(argv, *args, **kwargs):
        if argv[0] != "bun":
            return real(argv, *args, **kwargs)
        calls.append(list(argv))
        on_run()
        return subprocess.CompletedProcess(argv, 0)

    with patch.object(cp.subprocess, "run", side_effect=fake):
        yield calls


def update_error(
    apply_all: Callable[..., dict[str, str]], major: bool = False
) -> str | None:
    with patch.object(cp, "apply_all", apply_all), redirect_stdout(io.StringIO()):
        try:
            cp.update(major=major, to_tag=None, force=False, test=False)
        except cp.Fail as exc:
            return str(exc)
    return None


def applied(*_args, **_kwargs) -> dict[str, str]:
    return {"patch": "clean"}


def failing_apply(*_args, **_kwargs) -> dict[str, str]:
    raise cp.Fail("the Rex adapter's tests failed")


def update_stops_before_collies_updater_when_the_rex_patches_miss_the_target_release() -> (
    None
):
    with collie({REGISTRY: MOVED_REGISTRY}) as checkout, updater() as calls:
        head = sh(checkout, "git", "rev-parse", "HEAD")
        error = update_error(applied)
        assert error is not None and "do not apply to v1.0.1" in error, error
        assert "capture-rex --base v1.0.1" in error, error
        assert calls == [], f"Collie's updater ran: {calls}"
        assert sh(checkout, "git", "rev-parse", "HEAD") == head


def update_refuses_a_target_release_that_ships_its_own_rex_adapter() -> None:
    with collie({cp.REX_MARKER: "upstream\n"}), updater() as calls:
        error = update_error(applied)
        assert error is not None and "ships" in error, error
        assert calls == [], f"Collie's updater ran: {calls}"


def update_on_the_newest_release_still_checks_and_reaches_apply() -> None:
    apply_calls: list[int] = []

    def recording_apply(*_args, **_kwargs) -> dict[str, str]:
        apply_calls.append(1)
        return applied()

    with collie({"README": "1.0.1\n"}) as checkout, updater() as calls:
        sh(checkout, "git", "checkout", "-q", "--detach", "v1.0.1")
        assert cp.update_target(checkout, False, None) == "v1.0.1"
        error = update_error(recording_apply)
        assert error is None, error
        assert len(calls) == 1, calls
        assert apply_calls == [1], apply_calls


def the_update_target_is_the_newest_release_of_the_installed_major_or_the_next_with_major() -> (
    None
):
    with collie({"README": "1.0.1\n"}) as checkout:
        src = checkout.parent / "src"
        commit_release(src, "1.2.0", {}, annotated=True)
        commit_release(src, "1.3.0-beta.1", {})
        commit_release(src, "2.0.0", {})
        commit_release(src, "2.1.0", {}, annotated=True)
        commit_release(src, "3.0.0", {})
        sh(src, "git", "push", "-q", "--tags", str(checkout.parent / "origin.git"))
        assert cp.update_target(checkout, False, None) == "v1.2.0"
        assert cp.update_target(checkout, True, None) == "v2.1.0"
        assert cp.update_target(checkout, False, "v1.0.1") == "v1.0.1"
        assert cp.update_target(checkout, False, "1.0.1") == "v1.0.1"


def a_github_origin_reads_tags_over_anonymous_https() -> None:
    anonymous = "https::https://github.com/AltanS/collie.git"
    for url in (
        "git@github.com:AltanS/collie.git",
        "https://github.com/AltanS/collie",
        "ssh://git@github.com/AltanS/collie.git/",
        "https://www.GitHub.com/AltanS/collie.git\n",
    ):
        assert cp.anonymous_tag_url(url) == anonymous, url
    for url in (
        "/srv/origin.git",
        "https://gitlab.com/AltanS/collie.git",
        "https://github.com/AltanS",
        "",
    ):
        assert cp.anonymous_tag_url(url) is None, url


def a_failure_after_the_updater_moved_names_the_command_back_to_the_previous_release() -> (
    None
):
    with collie({"README": "1.0.1\n"}) as checkout:
        with updater(
            lambda: sh(checkout, "git", "checkout", "-q", "--detach", "v1.0.1")
        ):
            error = update_error(failing_apply)
        assert error is not None, "update reported success"
        assert "update --to-tag v1.0.0 --downgrade" in error, error
        assert "collie_patches.py apply" in error, error
        assert "best effort" in error, error
        assert str(cp.BACKUP_DIR) in error, error


def a_failure_after_a_major_crossing_offers_no_downgrade() -> None:
    with collie({"README": "1.0.1\n"}) as checkout:
        with updater(
            lambda: sh(checkout, "git", "checkout", "-q", "--detach", "v1.0.1")
        ):
            error = update_error(failing_apply, major=True)
        assert error is not None, "update reported success"
        assert "--downgrade" not in error, error


def an_outside_update_that_kept_the_rex_files_does_not_block_update() -> None:
    with collie({"README": "1.0.1\n"}) as checkout, updater() as calls:
        sh(checkout, "git", "apply", str(cp.REX_PATCH))
        sh(checkout, "git", "checkout", "--", ".")
        assert (checkout / cp.REX_MARKER).is_file()
        error = update_error(applied)
        assert error is None, error
        assert len(calls) == 1, calls


def capture_rex_records_against_the_given_base() -> None:
    with collie({REGISTRY: MOVED_REGISTRY}) as checkout:
        src = checkout.parent / "src"
        sh(src, "git", "checkout", "-q", "-b", cp.REX_BRANCH, "v1.0.1")
        (src / REGISTRY).write_text(
            MOVED_REGISTRY.replace('"zellij"]', '"zellij", "rex"]')
        )
        (src / cp.REX_MARKER).parent.mkdir(parents=True, exist_ok=True)
        (src / cp.REX_MARKER).write_text("export {};\n")
        (src / "bridge/mux/rex/adapter.test.ts").write_text("test();\n")
        sh(src, "git", "add", "-A")
        sh(src, "git", "commit", "-qm", "rex")
        with patch.object(cp, "REX_SOURCE", src), redirect_stdout(io.StringIO()):
            cp.capture_rex(base="v1.0.1")
        cp.check_rex_patches(checkout, "v1.0.1")


def rex_tests_run_in_a_fresh_private_tmpdir_that_is_removed_afterwards() -> None:
    seen: list[tuple[Path, int, int]] = []

    def fake_run(argv, cwd=None, env=None):
        tmp = Path(env["TMPDIR"])
        info = tmp.stat()
        seen.append((tmp, stat.S_IMODE(info.st_mode), info.st_uid))
        return subprocess.CompletedProcess(argv, 0, "", "")

    with (
        tempfile.TemporaryDirectory() as parent,
        patch.object(cp, "SHORT_TMP_PARENT", parent, create=True),
        patch.object(cp, "run", fake_run),
    ):
        cp.run_rex_tests(Path(parent))
        cp.run_rex_tests(Path(parent))
    (first, first_mode, first_uid), (second, second_mode, second_uid) = seen
    assert first != second, f"both runs shared {first}"
    assert first_mode == second_mode == 0o700, (oct(first_mode), oct(second_mode))
    assert first_uid == second_uid == os.getuid()
    assert not first.exists() and not second.exists(), "a test TMPDIR outlived its run"


def main() -> None:
    checks = [
        update_stops_before_collies_updater_when_the_rex_patches_miss_the_target_release,
        update_refuses_a_target_release_that_ships_its_own_rex_adapter,
        update_on_the_newest_release_still_checks_and_reaches_apply,
        the_update_target_is_the_newest_release_of_the_installed_major_or_the_next_with_major,
        a_github_origin_reads_tags_over_anonymous_https,
        a_failure_after_the_updater_moved_names_the_command_back_to_the_previous_release,
        a_failure_after_a_major_crossing_offers_no_downgrade,
        an_outside_update_that_kept_the_rex_files_does_not_block_update,
        capture_rex_records_against_the_given_base,
        rex_tests_run_in_a_fresh_private_tmpdir_that_is_removed_afterwards,
    ]
    failed = 0
    for check in checks:
        try:
            check()
        except Exception as e:
            failed += 1
            print(f"FAIL {check.__name__}: {type(e).__name__}: {e}")
    if failed:
        raise SystemExit(f"{failed}/{len(checks)} checks failed")
    print(f"ok ({len(checks)} checks)")


if __name__ == "__main__":
    main()
