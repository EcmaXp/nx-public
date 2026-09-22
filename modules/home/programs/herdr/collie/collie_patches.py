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

    ~/nx/modules/home/programs/herdr/collie/collie_patches.py status|apply|fonts|capture

A plugin update force-checks out the tree, so ``apply`` repeats after every one. It keeps
untracked files, so the patch only edits files upstream already tracks.
The saved ``web.patch`` is verified against Collie v1.8.0.

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
import zipfile
from pathlib import Path
from typing import Annotated

import httpx
import typer
from fontTools import subset
from fontTools.ttLib import TTFont
from toon_format import encode

HERE = Path(__file__).resolve().parent
PATCH = HERE / "web.patch"
FONT_CACHE = Path.home() / ".cache" / "collie-korean-mono"

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

    steps["patch"] = apply_patch(checkout)
    steps["fonts"] = ensure_fonts()
    install_fonts(checkout)

    if test:
        proc = run(["bun", "run", "test", "src/fonts.test.ts"], cwd=checkout / "web")
        if proc.returncode != 0:
            raise Fail(f"Collie's own tests rejected the result:\n{proc.stdout}")
        steps["tests"] = "passed"

    if restart:
        # `restart` does not rebuild: web/dist keeps the old CSS until `build` runs.
        collie_ctl(checkout, "build")
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
    print(encode(steps | live))


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
    checkout = checkout_dir()
    # Against HEAD so staged edits count; prefixes pinned because this file is committed.
    proc = run(
        [
            "git",
            "diff",
            "--no-ext-diff",
            "--src-prefix=a/",
            "--dst-prefix=b/",
            "HEAD",
            "--",
            *PATCHED_PATHS,
        ],
        cwd=checkout,
        env=git_env(checkout),
    )
    if proc.returncode != 0:
        raise Fail(f"git diff failed:\n{proc.stderr}")
    if not proc.stdout.strip():
        raise Fail(f"{'/'.join(PATCHED_PATHS)} is clean — there is nothing to record")
    PATCH.write_text(proc.stdout)
    print(encode({"patch": str(PATCH), "lines": str(len(proc.stdout.splitlines()))}))


def main() -> None:
    try:
        app()
    except Fail as exc:
        raise SystemExit(f"error: {exc}") from None


if __name__ == "__main__":
    main()
