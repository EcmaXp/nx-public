#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = [
#     "brotli>=1.1",
#     "fonttools[woff]>=4.55",
#     "httpx>=0.27",
#     "typer>=0.12",
#     "toon-format @ git+https://github.com/toon-format/toon-python.git@v0.9.0-beta.1",
# ]
# ///
"""Put Collie's local patches back after a plugin update wipes them.

    ~/nx/modules/home/programs/herdr/collie/collie_patches.py status|update|apply|fonts|capture

A plugin update force-checks out the tree, so ``apply`` repeats after every one. It keeps
untracked files, so the patch only edits files upstream already tracks. ``update`` runs
Collie's own updater and then ``apply``, and refuses while the checkout holds edits that
``web.patch`` lacks. The saved ``web.patch`` is verified against ``VERIFIED_VERSION``.

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
import zipfile
from datetime import datetime
from pathlib import Path
from typing import Annotated

import httpx
import typer
from fontTools import subset
from fontTools.ttLib import TTFont
from toon_format import encode

HERE = Path(__file__).resolve().parent
PATCH = HERE / "web.patch"
VERIFIED_VERSION = "v1.23.0"
FONT_CACHE = Path.home() / ".cache" / "collie-korean-mono"
BACKUP_DIR = Path.home() / ".cache" / "collie-patches"

PLUGIN_GLOB = "herdr.collie-*"
PLUGIN_PARENT = Path.home() / ".config/herdr/plugins/github"

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

BRIDGE = "http://127.0.0.1:8787"

app = typer.Typer(pretty_exceptions_enable=False, add_completion=False)


class Fail(Exception): ...


def run(
    argv: list[str], cwd: Path | None = None, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        argv, cwd=cwd, env=env, capture_output=True, text=True, check=False
    )


def checkout_dir() -> Path:
    """Herdr appends a hash to the directory, so it is found, never hardcoded."""
    found = sorted(PLUGIN_PARENT.glob(PLUGIN_GLOB))
    if not found:
        raise Fail(
            f"no Collie checkout under {PLUGIN_PARENT} — install it with:\n"
            "  herdr plugin install AltanS/collie --yes"
        )
    if len(found) > 1:
        raise Fail(
            f"several Collie checkouts, refusing to guess: {[str(p) for p in found]}"
        )
    return found[0]


def git_env(checkout: Path) -> dict[str, str]:
    return os.environ | {
        "GIT_WORK_TREE": str(checkout),
        "GIT_DIR": str(checkout / ".git"),
    }


def is_applied(checkout: Path) -> bool:
    css = checkout / "web/src/index.css"
    return css.is_file() and all(m in css.read_text() for m in APPLIED_MARKERS)


def fonts_present(checkout: Path) -> bool:
    fonts = checkout / "web/public/fonts"
    return all((fonts / name).is_file() for name in FONT_ASSETS)


def git_diff(checkout: Path, *paths: str) -> str:
    proc = run(
        [
            "git",
            "diff",
            "--no-ext-diff",
            "--src-prefix=a/",
            "--dst-prefix=b/",
            "HEAD",
            "--",
            *paths,
        ],
        cwd=checkout,
        env=git_env(checkout),
    )
    if proc.returncode != 0:
        raise Fail(f"git diff failed:\n{proc.stderr}")
    return proc.stdout


def installed_version(checkout: Path) -> str:
    proc = run(
        ["git", "describe", "--tags", "--always"],
        cwd=checkout,
        env=git_env(checkout),
    )
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
        bool, typer.Option(help="Update even when the checkout has unrecorded edits")
    ] = False,
    test: Annotated[
        bool, typer.Option(help="Run Collie's own tests for the patched files")
    ] = True,
) -> None:
    """Run Collie's updater, then re-apply the patch, rebuild, and verify."""
    checkout = checkout_dir()
    before = installed_version(checkout)

    if (
        not force
        and is_applied(checkout)
        and git_diff(checkout, *PATCHED_PATHS) != PATCH.read_text()
    ):
        raise Fail(
            f"the checkout holds edits {PATCH.name} lacks, and the update would discard them.\n"
            f"Record them first with `{Path(__file__).name} capture`, or pass --force."
        )

    BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    backup = BACKUP_DIR / f"{before}-{datetime.now():%Y%m%dT%H%M%S}.patch"
    backup.write_text(git_diff(checkout))

    argv = ["bun", "run", "cli/main.ts", "update"]
    if major:
        argv.append("--major")
    if to_tag:
        argv += ["--to-tag", to_tag]
    code = subprocess.run(argv, cwd=checkout, check=False).returncode
    after = installed_version(checkout)
    if code != 0 and after == before:
        raise Fail(f"Collie's updater exited {code} and the checkout did not move")

    result = {
        "before": before,
        "after": after,
        "backup": str(backup),
        "updater": f"exit {code}" if code == 0 else f"exit {code} (recovered by apply)",
        **apply_all(checkout, restart=True, test=test),
    }
    if result["patch"] == "3way" or after != VERIFIED_VERSION:
        result["next"] = "capture, bump VERIFIED_VERSION, commit"
    print(encode(result))


def apply_all(checkout: Path, restart: bool, test: bool) -> dict[str, str]:
    steps: dict[str, str] = {}
    steps["patch"] = apply_patch(checkout)
    steps["fonts"] = ensure_fonts()
    install_fonts(checkout)

    if test:
        proc = run(["bun", "run", "test", *PATCHED_TESTS], cwd=checkout / "web")
        if proc.returncode != 0:
            raise Fail(f"Collie's own tests rejected the result:\n{proc.stdout}")
        steps["tests"] = "passed"

    if restart:
        # `restart` does not rebuild: web/dist keeps the old CSS until `build` runs.
        resign_binary(checkout)
        collie_ctl(checkout, "build")
        steps["signature"] = resign_binary(checkout)
        collie_ctl(checkout, "restart")
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
