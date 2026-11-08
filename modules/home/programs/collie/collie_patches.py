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
"""Put Collie's local patches back after an update replaces the tree.

    ~/nx/modules/home/programs/collie/collie_patches.py status|update|apply|fonts|capture

An update checks out a fresh tree, so ``apply`` repeats after every one. It keeps
untracked files, so ``web.patch`` only edits files upstream already tracks, and ``apply`` sets
aside the files the Rex patches create before applying them again. ``rex.patch`` is the Rex
adapter's shipped code and ``rex-tests.patch`` its tests, fixtures and recorded captures, both
recorded from a branch of the dev clone with ``capture-rex``. ``update`` runs
Collie's own updater and then ``apply``, and refuses while the checkout holds edits that
``web.patch`` lacks, or while the Rex patches miss the release it would take. The saved ``web.patch`` is verified against ``VERIFIED_VERSION``.

Each of these is measured on an iPhone and breaks if tidied:

- D2Coding sits directly after JetBrains Mono, which makes a Hangul syllable exactly two
  cells with JetBrains Mono present and without it. Anywhere else in the stack, Hangul
  falls to the proportional system face and every column after a Korean word slides.
- ``:focus`` holds opacity 0 for 0.02s with ``step-end``. WebKit skips its reveal pan only
  for a field it sees fully transparent: 0.1, 0.5 and 0.99 all pan, a linear fade is
  transparent at t=0 alone and fails intermittently, and anything shorter than 0.02s
  misses a session's first focus.
- The shell shrinks by ``--app-kb`` and never moves its top edge. Compensating the pan
  instead chases the compositor a frame behind, which reads as a sliding header.
- ``--app-h`` stays ``100dvh`` from the Home Screen too. The ``black`` status bar starts the
  page below the bar, where ``100dvh`` is the full height; upstream's ``100lvh`` assumes a
  page under the bar and pushes the tab bar a status bar's height off the screen.
- A hardware keyboard's pill sits over the bottom 58.3pt and iOS resizes nothing for it, so
  the shell sets aside ``KEYBOARD_PILL_PX`` while an editable field has focus.
  ``document.hasFocus()`` releases it when iOS takes text input from a field that keeps
  DOM focus (no caret, no pill), which otherwise leaves an empty band.
- ``body`` carries the iOS bar colour, because iOS paints its own chrome from BODY and
  reads neither ``html`` nor the manifest. The shell repaints the app's own ground.
- Conjoining Jamo is absent from D2Coding, so it stays out of the subset range. A range
  wider than the font falls through to the proportional face, which is the original bug.

A new D2Coding release means bumping the version below AND the filenames the patch lists
in ``index.css`` and ``sw-routes.ts``.
"""

from __future__ import annotations

import io
import os
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile
from datetime import datetime
from pathlib import Path
from typing import Annotated

import httpx
import typer
from fontTools import subset
from fontTools.ttLib import TTFont
from pydantic import BaseModel, ValidationError
from toon_format import encode

HERE = Path(__file__).resolve().parent
PATCH = HERE / "web.patch"
VERIFIED_VERSION = "v1.26.1"
FONT_CACHE = Path.home() / ".cache" / "collie-korean-mono"
BACKUP_DIR = Path.home() / ".cache" / "collie-patches"

COLLIE_ROOT = Path(os.environ.get("COLLIE_DIR") or Path.home() / ".local/share/collie")
COLLIE_REPO = "https://github.com/AltanS/collie.git"

D2CODING_VERSION = "1.3.2"
D2CODING_STAMP = "20180524"
D2CODING_ZIP = (
    f"https://github.com/naver/d2codingfont/releases/download/VER{D2CODING_VERSION}"
    f"/D2Coding-Ver{D2CODING_VERSION}-{D2CODING_STAMP}.zip"
)
D2CODING_LICENSE = "https://raw.githubusercontent.com/naver/d2codingfont/master/OFL.txt"
LICENSE_NAME = "D2Coding-LICENSE.txt"

SUBSETS = {
    "text": "U+0020-007E,U+00A0-00FF,U+0100-017F,U+0370-03FF,U+0400-04FF,U+2000-206F,"
    "U+2070-209F,U+20A0-20BF,U+2100-214F,U+2190-21FF,U+2200-22FF,U+2300-23FF,"
    "U+2500-257F,U+2580-259F,U+25A0-25FF,U+2600-26FF,U+2700-27BF",
    "hangul": "U+3130-318F,U+AC00-D7A3",
}

FONT_ASSETS = (
    *(f"d2coding-{D2CODING_VERSION}-{name}.woff2" for name in SUBSETS),
    LICENSE_NAME,
)

# `git apply` is atomic and this file is what Collie's build reads, so it settles it.
APPLIED_MARKERS = ('font-family: "D2Coding"', "--app-kb")

PATCHED_PATHS = ("web/src", "web/index.html")
PATCHED_TESTS = ("src/fonts.test.ts", "src/hooks/use-keyboard.test.ts")

REX_PATCH = HERE / "rex.patch"
REX_TEST_PATCH = HERE / "rex-tests.patch"
REX_PATCHES = (REX_PATCH, REX_TEST_PATCH)
REX_SOURCE = Path.home() / "src/collie"
REX_BRANCH = "feat/rex-adapter"
REX_PATHS = ("bridge", "cli")
REX_TEST_PATHS = (
    "*.test.ts",
    "bridge/mux/fixtures.ts",
    "bridge/mux/rex/fixture.ts",
    "bridge/mux/rex/captures",
)
REX_MARKER = "bridge/mux/rex/adapter.ts"
REX_TESTS = (
    "bridge/journal/claude.test.ts",
    "bridge/journal/claude-pid.test.ts",
    "bridge/mux/rex",
    "bridge/mux/conformance.test.ts",
    "bridge/mux/registry.test.ts",
    "bridge/mux/keys.test.ts",
    "bridge/mux/logo.test.ts",
    "bridge/state-engine-unwatched.test.ts",
    "bridge/surface/shadow.test.ts",
    "cli/mux-list.test.ts",
)
SHORT_TMP_PARENT = "/private/tmp" if sys.platform == "darwin" else "/tmp"

RELEASE_REF = re.compile(r"\trefs/tags/v(\d+)\.(\d+)\.(\d+)$")
MANIFEST_VERSION = re.compile(r'^version[ \t]*=[ \t]*"([^"]*)"', re.M)

BRIDGE = "http://127.0.0.1:8787"

app = typer.Typer(pretty_exceptions_enable=False, add_completion=False)


class Fail(Exception): ...


class InstanceStatus(BaseModel):
    service: str = ""


def run(
    argv: list[str], cwd: Path | None = None, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        argv, cwd=cwd, env=env, capture_output=True, text=True, check=False
    )


def checkout_dir() -> Path:
    """The live tree: ``current`` once Collie's updater has flipped a version, else the clone."""
    current = COLLIE_ROOT / "current"
    if current.is_symlink():
        return current.resolve()
    if (COLLIE_ROOT / ".git").exists():
        return COLLIE_ROOT
    raise Fail(
        f"no standalone Collie at {COLLIE_ROOT}, clone it with:\n"
        f"  git clone {COLLIE_REPO} {COLLIE_ROOT}"
    )


def git_env(checkout: Path) -> dict[str, str]:
    return os.environ | {
        "GIT_WORK_TREE": str(checkout),
        "GIT_DIR": str(checkout / ".git"),
    }


def git(repo: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return run(["git", *args], cwd=repo, env=git_env(repo))


def write_backup(name: str, text: str) -> Path:
    BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    backup = BACKUP_DIR / f"{name}-{datetime.now():%Y%m%dT%H%M%S}.patch"
    backup.write_text(text)
    return backup


def is_applied(checkout: Path) -> bool:
    css = checkout / "web/src/index.css"
    return css.is_file() and all(m in css.read_text() for m in APPLIED_MARKERS)


def fonts_present(checkout: Path) -> bool:
    fonts = checkout / "web/public/fonts"
    return all((fonts / name).is_file() for name in FONT_ASSETS)


def git_diff(repo: Path, *paths: str, rev: str = "HEAD") -> str:
    proc = git(
        repo,
        "diff",
        "--no-ext-diff",
        "--src-prefix=a/",
        "--dst-prefix=b/",
        rev,
        "--",
        *paths,
    )
    if proc.returncode != 0:
        raise Fail(f"git diff in {repo} failed:\n{proc.stderr}")
    return proc.stdout


def without_index_lines(diff: str) -> str:
    """Drop `index` lines: git lengthens abbreviated blob ids as the repo grows."""
    return re.sub(r"^index [0-9a-f]+\.\.[0-9a-f]+.*\n", "", diff, flags=re.M)


def installed_version(checkout: Path) -> str:
    proc = git(checkout, "describe", "--tags", "--always")
    if proc.returncode != 0:
        raise Fail(f"git describe failed:\n{proc.stderr}")
    return proc.stdout.strip()


def fetch(url: str, timeout: float) -> bytes:
    response = httpx.get(url, follow_redirects=True, timeout=timeout)
    if response.status_code != 200:
        raise Fail(f"{url} answered {response.status_code}")
    return response.content


def build_fonts() -> None:
    with zipfile.ZipFile(io.BytesIO(fetch(D2CODING_ZIP, 120.0))) as archive:
        # The plain regular: a ligature would fuse glyphs emitted as separate cells.
        ttf = archive.read(
            f"D2Coding/D2Coding-Ver{D2CODING_VERSION}-{D2CODING_STAMP}.ttf"
        )

    FONT_CACHE.mkdir(parents=True, exist_ok=True)
    for name, unicodes in SUBSETS.items():
        font = TTFont(io.BytesIO(ttf))
        subsetter = subset.Subsetter(
            options=subset.Options(
                layout_features=[], hinting=False, desubroutinize=True
            )
        )
        subsetter.populate(unicodes=subset.parse_unicodes(unicodes))
        subsetter.subset(font)
        font.flavor = "woff2"
        font.save(str(FONT_CACHE / f"d2coding-{D2CODING_VERSION}-{name}.woff2"))

    (FONT_CACHE / LICENSE_NAME).write_bytes(fetch(D2CODING_LICENSE, 30.0))


def ensure_fonts() -> str:
    if all((FONT_CACHE / name).is_file() for name in FONT_ASSETS):
        return "cache"
    build_fonts()
    return "built"


def install_fonts(checkout: Path) -> None:
    dest = checkout / "web/public/fonts"
    dest.mkdir(parents=True, exist_ok=True)
    for name in FONT_ASSETS:
        shutil.copy2(FONT_CACHE / name, dest / name)


def apply_patch(checkout: Path) -> str:
    if is_applied(checkout):
        return "already-applied"
    env = git_env(checkout)
    # cwd, not just the env: `git apply` resolves paths against it and would otherwise
    # write into the wrong repo, exit 0. --3way first: blob ids survive upstream drift.
    for args, label in ((["--3way"], "3way"), ([], "clean")):
        proc = run(["git", "apply", *args, str(PATCH)], cwd=checkout, env=env)
        if proc.returncode == 0:
            if not is_applied(checkout):
                raise Fail(
                    f"git apply reported success but {checkout} is unchanged — "
                    "the patch landed somewhere else"
                )
            # --3way stages what it merged, and capture()'s plain diff would miss it.
            run(["git", "reset", "-q", "--", *PATCHED_PATHS], cwd=checkout, env=env)
            return label
    raise Fail(
        f"{PATCH.name} no longer applies to this Collie release.\n"
        "Redo the edits by hand against the new upstream, then re-record them:\n"
        f"  {Path(__file__).name} capture"
    )


def upstream_ships_rex(checkout: Path, rev: str = "HEAD") -> bool:
    return git(checkout, "cat-file", "-e", f"{rev}:{REX_MARKER}").returncode == 0


def patch_applied(checkout: Path, patch: Path) -> bool:
    return (
        patch.is_file()
        and git(checkout, "apply", "--reverse", "--check", str(patch)).returncode == 0
    )


def patch_paths(checkout: Path, patch: Path) -> tuple[list[str], list[str]]:
    proc = git(checkout, "apply", "--numstat", "--summary", str(patch))
    if proc.returncode != 0:
        raise Fail(f"git apply --numstat rejected {patch.name}:\n{proc.stderr}")
    lines = proc.stdout.splitlines()
    created = [
        line.split()[-1] for line in lines if line.strip().startswith("create mode")
    ]
    touched = [line.split("\t")[-1] for line in lines if "\t" in line]
    return [rel for rel in touched if rel not in created], created


def tracked_paths(checkout: Path, paths: list[str]) -> set[str]:
    if not paths:
        return set()
    proc = git(checkout, "ls-files", "--", *paths)
    if proc.returncode != 0:
        raise Fail(f"git ls-files failed:\n{proc.stderr}")
    return set(proc.stdout.splitlines())


def restore_tracked(checkout: Path, patch: Path, paths: list[str]) -> None:
    known = tracked_paths(checkout, paths)
    tracked = [rel for rel in paths if rel in known]
    if not tracked:
        return
    diff = git_diff(checkout, *tracked)
    if diff.strip():
        write_backup(f"{patch.stem}-tracked", diff)
    proc = git(checkout, "checkout", "HEAD", "--", *tracked)
    if proc.returncode != 0:
        raise Fail(f"could not restore the files {patch.name} edits:\n{proc.stderr}")


def set_aside_untracked(checkout: Path, patch: Path, paths: list[str]) -> list[str]:
    present = [rel for rel in paths if (checkout / rel).exists()]
    upstream = tracked_paths(checkout, present)
    if upstream:
        raise Fail(
            f"upstream tracks {', '.join(sorted(upstream))} now, and {patch.name} would create it"
        )
    stash = BACKUP_DIR / f"{patch.stem}-untracked-{datetime.now():%Y%m%dT%H%M%S}"
    for rel in present:
        (stash / rel).parent.mkdir(parents=True, exist_ok=True)
        shutil.move(checkout / rel, stash / rel)
    return present


def apply_rex(checkout: Path, patch: Path) -> str:
    if not patch.is_file():
        return f"skip (no {patch.name})"
    if upstream_ships_rex(checkout):
        raise Fail(
            f"upstream Collie ships {REX_MARKER} now. Compare it with {REX_PATCH.name}, "
            "then drop the patches rather than applying them over upstream's adapter."
        )
    if patch_applied(checkout, patch):
        return "already-applied"
    edited, created = patch_paths(checkout, patch)
    restore_tracked(checkout, patch, edited)
    moved = set_aside_untracked(checkout, patch, created)
    proc = git(checkout, "apply", str(patch))
    if proc.returncode != 0 or not patch_applied(checkout, patch):
        raise Fail(
            f"{patch.name} no longer applies to this Collie release:\n{proc.stderr}\n"
            f"Rebase {REX_BRANCH} in {REX_SOURCE} onto the new tag, then:\n"
            f"  {Path(__file__).name} capture-rex"
        )
    return f"applied (set aside {len(moved)} stale files)" if moved else "applied"


def run_rex_tests(checkout: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="collie-", dir=SHORT_TMP_PARENT) as tmp:
        proc = run(
            ["bun", "test", *REX_TESTS],
            cwd=checkout,
            env=os.environ | {"TMPDIR": tmp},
        )
    if proc.returncode != 0:
        raise Fail(
            f"the Rex adapter's tests failed on the patched checkout:\n{proc.stdout[-4000:]}\n{proc.stderr[-4000:]}"
        )


def manifest_version(checkout: Path) -> str | None:
    """The herdr-plugin.toml version, read with upstream's line regex (bridge/version.ts)."""
    manifest = checkout / "herdr-plugin.toml"
    if not manifest.is_file():
        return None
    found = MANIFEST_VERSION.search(manifest.read_text(errors="replace"))
    return found[1] if found and found[1] else None


def anonymous_tag_url(url: str) -> str | None:
    """Upstream's anonymousTagUrl: a GitHub origin read over HTTPS that no insteadOf rule rewrites."""
    raw = url.strip()
    if re.match(r"[a-zA-Z][a-zA-Z0-9+.-]*://", raw):
        rest = re.sub(r"^[^@/]+@", "", raw.split("://", 1)[1])
    elif scp := re.match(r"(?:[^@/]+@)?([^:/]+):(.+)$", raw):
        rest = f"{scp[1]}/{scp[2]}"
    else:
        return None
    host, _, path = rest.partition("/")
    parts = [part for part in path.rstrip("/").removesuffix(".git").split("/") if part]
    if host.lower().removeprefix("www.") != "github.com" or len(parts) != 2:
        return None
    return f"https::https://github.com/{parts[0]}/{parts[1]}.git"


def tag_remote(checkout: Path) -> str:
    proc = git(checkout, "remote", "get-url", "origin")
    return anonymous_tag_url(proc.stdout if proc.returncode == 0 else "") or "origin"


def update_target(checkout: Path, major: bool, to_tag: str | None) -> str | None:
    """The release Collie's updater takes on the Stable line: the newest of this major, or of the next with --major."""
    if to_tag:
        return to_tag if to_tag.startswith("v") else f"v{to_tag}"
    proc = git(checkout, "ls-remote", "--tags", tag_remote(checkout))
    if proc.returncode != 0:
        raise Fail(f"could not list Collie's release tags:\n{proc.stderr}")
    releases = sorted(
        (int(m[1]), int(m[2]), int(m[3]))
        for m in map(RELEASE_REF.search, proc.stdout.splitlines())
        if m
    )
    installed = re.match(r"(\d+)\.", manifest_version(checkout) or "")
    if installed is not None:
        current = int(installed[1])
        want = (
            min((v[0] for v in releases if v[0] > current), default=None)
            if major
            else current
        )
        releases = [v for v in releases if v[0] == want]
    return "v{}.{}.{}".format(*releases[-1]) if releases else None


def check_rex_patches(checkout: Path, tag: str) -> None:
    patches = [str(patch) for patch in REX_PATCHES if patch.is_file()]
    ref = f"refs/tags/{tag}"
    shallow = git(checkout, "rev-parse", "--is-shallow-repository").stdout.strip()
    depth = ["--depth", "1"] if shallow == "true" else []
    fetched = git(
        checkout, "fetch", "-q", *depth, tag_remote(checkout), f"+{ref}:{ref}"
    )
    if fetched.returncode != 0:
        raise Fail(f"could not fetch {tag} to check the Rex patches:\n{fetched.stderr}")
    if upstream_ships_rex(checkout, tag):
        raise Fail(
            f"upstream Collie {tag} ships {REX_MARKER}. Compare it with {REX_PATCH.name}, "
            "then drop the patches rather than updating over upstream's adapter."
        )
    with tempfile.TemporaryDirectory() as tmp:
        env = git_env(checkout) | {"GIT_INDEX_FILE": str(Path(tmp) / "index")}
        proc = run(["git", "read-tree", tag], cwd=checkout, env=env)
        if proc.returncode == 0:
            proc = run(
                ["git", "apply", "--cached", "--check", *patches], cwd=checkout, env=env
            )
    if proc.returncode != 0:
        raise Fail(
            f"the Rex patches do not apply to {tag}, so the update stopped before touching Collie:\n"
            f"{proc.stderr}Rebase {REX_BRANCH} in {REX_SOURCE} onto {tag}, then:\n"
            f"  {Path(__file__).name} capture-rex --base {tag}\n"
            f"  {Path(__file__).name} update"
        )


def instance_status(checkout: Path) -> InstanceStatus:
    proc = run([str(checkout / "bin/collie"), "status", "--json"], cwd=checkout)
    try:
        return InstanceStatus.model_validate_json(proc.stdout)
    except ValidationError:
        return InstanceStatus()


def restart_instance(checkout: Path) -> str:
    for _ in range(2):
        run([str(checkout / "bin/collie"), "restart"], cwd=checkout)
        service = instance_status(checkout).service
        if service.startswith("launchd (") and "active (pid" in service:
            return "supervised"
        if service.startswith("systemd --user (") and service.endswith("· active"):
            return "supervised"
    raise Fail(f"Collie is not supervised after two restarts: {service}")


def collie_ctl(checkout: Path, command: str) -> str:
    proc = run(["bash", str(checkout / "scripts/collie-ctl.sh"), command], cwd=checkout)
    if proc.returncode != 0:
        raise Fail(f"collie-ctl.sh {command} failed:\n{proc.stdout}\n{proc.stderr}")
    return proc.stdout.strip()


def resign_binary(checkout: Path) -> str:
    """Nix's bun leaves `--compile` output with a broken signature, which macOS SIGKILLs."""
    binary = checkout / "bin/collie"
    if sys.platform != "darwin" or not binary.is_file():
        return "skip"
    if run(["codesign", "-v", str(binary)]).returncode == 0:
        return "valid"
    proc = run(["codesign", "--force", "--sign", "-", str(binary)])
    if proc.returncode != 0 or run(["codesign", "-v", str(binary)]).returncode != 0:
        raise Fail(f"could not re-sign {binary}:\n{proc.stderr}")
    return "re-signed"


def serving() -> dict[str, str]:
    try:
        index = httpx.get(f"{BRIDGE}/", timeout=5.0).text
    except httpx.HTTPError as exc:
        return {"bridge": f"unreachable ({exc.__class__.__name__})"}

    # Anchored on the extension: index.html names the JS bundle first.
    def asset(pattern: str) -> tuple[str, str]:
        match = re.search(pattern, index)
        url = match.group(0) if match else ""
        return url, httpx.get(f"{BRIDGE}{url}", timeout=5.0).text if url else ""

    css_url, css = asset(r"/assets/index-[\w-]+\.css")
    _, js = asset(r"/assets/index-[\w-]+\.js")
    # The minifier owns quotes and spacing; normalise before matching on order.
    flat = css.replace('"', "").replace(", ", ",")

    out = {
        "css": css_url or "not found",
        "order_ok": str("JetBrains Mono,D2Coding" in flat),
        "faces": str(css.count("font-family:D2Coding")),
        # Minifiers rename neither a custom property nor a DOM member.
        "viewport_ok": str("--app-kb" in css and "visualViewport" in js),
    }
    for name in FONT_ASSETS:
        if not name.endswith(".woff2"):
            continue
        head = httpx.head(f"{BRIDGE}/fonts/{name}", timeout=5.0)
        out[name.removeprefix("d2coding-1.3.2-").removesuffix(".woff2")] = (
            f"{head.status_code} {head.headers.get('content-type', '?')}"
        )
    return out


@app.command()
def status() -> None:
    """Report where the checkout is and whether the patch is live."""
    checkout = checkout_dir()
    print(
        encode(
            {
                "checkout": str(checkout),
                "patched": str(is_applied(checkout)),
                "fonts_in_checkout": str(fonts_present(checkout)),
                "fonts_cached": str(
                    all((FONT_CACHE / n).is_file() for n in FONT_ASSETS)
                ),
                **serving(),
            }
        )
    )


@app.command()
def apply(
    dry_run: Annotated[bool, typer.Option("-n", "--dry-run")] = False,
    restart: Annotated[
        bool, typer.Option(help="Rebuild the UI and restart the bridge")
    ] = True,
    test: Annotated[
        bool, typer.Option(help="Run Collie's own tests for the patched files")
    ] = True,
) -> None:
    """Re-apply the patch, rebuild, and prove the bridge serves it."""
    checkout = checkout_dir()
    steps = {
        "checkout": str(checkout),
        "patch": "skip (already applied)" if is_applied(checkout) else "git apply",
        **{
            patch.stem: "skip (already applied)"
            if patch_applied(checkout, patch)
            else "git apply"
            for patch in REX_PATCHES
        },
        "fonts": "cache"
        if all((FONT_CACHE / n).is_file() for n in FONT_ASSETS)
        else "build",
        "rebuild": str(restart),
    }
    if dry_run:
        print(encode(steps))
        return
    print(encode(steps | apply_all(checkout, restart=restart, test=test)))


@app.command()
def update(
    major: Annotated[
        bool, typer.Option(help="Allow a new major release (Collie's --major)")
    ] = False,
    to_tag: Annotated[
        str | None, typer.Option(help="Take this exact release, like v1.16.0")
    ] = None,
    force: Annotated[
        bool,
        typer.Option(
            help="Update even when the checkout has unrecorded edits or the Rex patches miss the release"
        ),
    ] = False,
    test: Annotated[
        bool, typer.Option(help="Run Collie's own tests for the patched files")
    ] = True,
) -> None:
    """Run Collie's updater, then re-apply the patch, rebuild, and verify."""
    checkout = checkout_dir()
    before = installed_version(checkout)
    before_version = manifest_version(checkout)

    if (
        not force
        and is_applied(checkout)
        and without_index_lines(git_diff(checkout, *PATCHED_PATHS))
        != without_index_lines(PATCH.read_text())
    ):
        raise Fail(
            f"the checkout holds edits {PATCH.name} lacks, and the update would discard them.\n"
            f"Record them first with `{Path(__file__).name} capture`, or pass --force."
        )
    if not force and any(patch.is_file() for patch in REX_PATCHES):
        target = update_target(checkout, major, to_tag)
        if target is not None:
            check_rex_patches(checkout, target)

    backup = write_backup(before, git_diff(checkout))

    argv = ["bun", "run", "cli/main.ts", "update"]
    if major:
        argv.append("--major")
    if to_tag:
        argv += ["--to-tag", to_tag]
    code = subprocess.run(argv, cwd=checkout, check=False).returncode
    checkout = checkout_dir()
    after = installed_version(checkout)
    if code != 0 and after == before:
        raise Fail(f"Collie's updater exited {code} and the checkout did not move")

    try:
        applied = apply_all(checkout, restart=True, test=test)
    except Fail as exc:
        if after == before or before_version is None or major:
            raise
        raise Fail(
            f"{exc}\nThe update moved Collie from {before} to {after}. To try v{before_version} again "
            f"(best effort: the patches are recorded against {VERIFIED_VERSION} and may not apply there):\n"
            f"  (cd {checkout} && bun run cli/main.ts update --to-tag v{before_version} --downgrade)\n"
            f"  {Path(__file__).name} apply\n"
            f"Tracked edits from before the update: {backup}"
        ) from None
    result = {
        "before": before,
        "after": after,
        "backup": str(backup),
        "updater": f"exit {code}" if code == 0 else f"exit {code} (recovered by apply)",
        **applied,
    }
    if result["patch"] == "3way" or after != VERIFIED_VERSION:
        result["next"] = "capture, bump VERIFIED_VERSION, commit"
    print(encode(result))


def apply_all(checkout: Path, restart: bool, test: bool) -> dict[str, str]:
    steps: dict[str, str] = {}
    steps["patch"] = apply_patch(checkout)
    for patch in REX_PATCHES:
        steps[patch.stem] = apply_rex(checkout, patch)
    steps["fonts"] = ensure_fonts()
    install_fonts(checkout)

    if test:
        proc = run(["bun", "run", "test", *PATCHED_TESTS], cwd=checkout / "web")
        if proc.returncode != 0:
            raise Fail(f"Collie's own tests rejected the result:\n{proc.stdout}")
        if REX_TEST_PATCH.is_file():
            run_rex_tests(checkout)
        steps["tests"] = "passed"

    if restart:
        # `restart` does not rebuild: web/dist keeps the old CSS until `build` runs.
        resign_binary(checkout)
        collie_ctl(checkout, "build")
        steps["signature"] = resign_binary(checkout)
        steps["restart"] = restart_instance(checkout)
        steps["rebuild"] = "built + restarted"

    live = serving()
    # Collie's own test only reads the checkout; only the served CSS settles it.
    if (
        restart
        and "order_ok" in live
        and (live["order_ok"], live["faces"], live["viewport_ok"])
        != ("True", "2", "True")
    ):
        raise Fail(f"the bridge is not serving the patched CSS:\n{encode(live)}")
    return steps | live


@app.command()
def fonts() -> None:
    """Rebuild the D2Coding subsets from upstream into the cache."""
    build_fonts()
    print(
        encode({name: str((FONT_CACHE / name).stat().st_size) for name in FONT_ASSETS})
    )


@app.command(name="capture-rex")
def capture_rex(
    base: Annotated[
        str | None,
        typer.Option(help="Record against this release instead of the installed one"),
    ] = None,
) -> None:
    """Re-record rex.patch and rex-tests.patch from the dev clone's Rex branch, against the installed tag."""
    base = base or installed_version(checkout_dir())
    rev = f"{base}...{REX_BRANCH}"
    shipped = git_diff(
        REX_SOURCE, *REX_PATHS, *(f":(exclude){p}" for p in REX_TEST_PATHS), rev=rev
    )
    tests = git_diff(REX_SOURCE, *REX_TEST_PATHS, rev=rev)
    if REX_MARKER not in shipped:
        raise Fail(f"{REX_BRANCH} adds no {REX_MARKER} against {base}")
    REX_PATCH.write_text(shipped)
    REX_TEST_PATCH.write_text(tests)
    print(
        encode(
            {
                "base": base,
                REX_PATCH.name: str(len(shipped.splitlines())),
                REX_TEST_PATCH.name: str(len(tests.splitlines())),
            }
        )
    )


@app.command()
def capture() -> None:
    """Re-record the patch from the live checkout, after editing it by hand."""
    diff = git_diff(checkout_dir(), *PATCHED_PATHS)
    if not diff.strip():
        raise Fail(f"{'/'.join(PATCHED_PATHS)} is clean — there is nothing to record")
    PATCH.write_text(diff)
    print(encode({"patch": str(PATCH), "lines": str(len(diff.splitlines()))}))


def main() -> None:
    try:
        app()
    except Fail as exc:
        raise SystemExit(f"error: {exc}") from None


if __name__ == "__main__":
    main()
