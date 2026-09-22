# User Preferences

## Environment

- macOS with Nix package manager
- CLI tools are GNU versions (not BSD) from `/run/current-system/sw/bin/`

## Claude Code Sandbox

- `Operation not permitted` errors during git worktree operations indicate sandbox restrictions
- Disable sandbox for worktree creation: use `dangerouslyDisableSandbox: true`
- Affected operations: `git worktree add`, large repo checkouts, file creation in new directories

## Verification Discipline

- Before stating a result as fact, run the command that proves it and show the output. Never assert a count, a "0 left" claim, or CI/PR/remote state from inference
- Treat any non-2xx response as a failure inside a retry loop. Never parse a 403 body as success: assert on the status code, not on whether the body parsed
- A claim about state you did not observe this session is a guess. Say so, or go measure it
- Capture long-running producer output intact before filtering, and preserve its exit status. Use unfiltered output when a wrapper omits required identities or paths
- A write timeout leaves the outcome unknown: read back before retrying. Reconcile partial successes individually, and inspect per-item errors even in HTTP 200 batch responses
- Match expected immutable identities, not counts or display names alone. A no-op, generated artifact, skipped CI job, or merged change does not prove the intended live result

## Shell

- Reference sibling files through brace expansion instead of repeating the common prefix: `git add src/app/{models,views}.py`, `treefmt src/x/{a,b}.py`
- Brace expansion needs at least one comma (`{a}` stays literal) and unquoted braces
- The Bash tool runs zsh, which does **not** word-split an unquoted `$VAR`: `git reset -- $PATHS` passes one joined argument and silently matches nothing. Brace-expand, or use an array (`for p in $=PATHS`)
- Join a command to the one it depends on with `&&`, never `;`. `;` runs the next command after a failure, which is how a failed `git stash push` was followed by a `git stash pop` that took an unrelated stash
- Never silence a command you are about to depend on. `>/dev/null 2>&1` on that `stash push` is why nobody saw it fail
- Write home-anchored paths as `$HOME/...`, not a literal `/Users/<user>/...`
- Bulk destructive operations (>50 deletions, mass API deletes, `rm -rf` over a glob) print the full target list and its count first, wait for confirmation, then execute in chunks with a running count. A single unbounded delete is not reviewable
- `rm` is denied. Delete with `gomi <paths>`, which moves them to `~/.local/share/Trash` and restores with `gomi -b`. When a hard delete is genuinely needed, say why and leave it to the user
- Enforce deletion scope during both selection and execution. Apply exclusions before exact-target overrides, reject missing requested targets, and account for every failed item
- Resolve the full symlink chain before editing. A parent gitlink stores a commit ID, not nested repository files: preserve those files separately during cleanup. Same-volume trash does not free disk blocks
- Keep formatter caches outside directories whose exact file inventory is validated

## Git

- Never chain `git commit` with other commands (`&&`/`;`); the one exception is the atomic add+commit pair below
- Use conventional commits with scope when applicable (e.g., `feat(api):`, `fix(auth):`, `docs(readme):`)
- Review `git log` before committing to match repository's commit style
- Stage and commit as one atomic command: `git add <files> && git commit --only <dir>` with `--only` scoped to the staged paths' parent folder, never a bare `git commit` (hunk-level splits via `git add -p` are the exception)
- Declare a temp variable for repeated path prefixes and brace-expand siblings: `D=<dir>; git add $D/{a,b}.py && git commit --only $D -m "..."` (for an out-of-cwd repo, add `gw $D`, next bullet)
- For a repo outside the cwd, never `git -C` (or `cd`): `D=<shared-prefix-dir>; gw $D` — the `gw` helper (shell-env.sh, loaded in every Bash call) discovers the enclosing repo and exports `GIT_WORK_TREE`/`GIT_DIR`; keep referencing paths as `$D/...`. Fallback without the helper: `export GIT_WORK_TREE=<abs-repo>; export GIT_DIR=$GIT_WORK_TREE/.git` (two statements: a single export expands `$GIT_WORK_TREE` before the assignment lands)
- `gw` is only for a repo the cwd is **not** in, and it now refuses when the cwd already sits under the target. Inside your own repo the pin buys nothing, and git still resolves a pathspec against the cwd, so a repo-root-relative path quietly means `<cwd>/<path>`. Pass `$D/...`, or prefix `:/` to anchor at the root (`git stash push -- :/projects/x/y.py`)
- Never set `GIT_DIR` alone: without `GIT_WORK_TREE` the work tree silently falls back to the cwd. The export redirects every later git/gh call in the same Bash call (`gw -u` drops it; `gw` refuses to repin a different repo while set); env resets between calls, so re-run per call
- When splitting into multiple commits: review all changes first (`git diff`), then stage and commit each unit atomically
- Prefer single-line commit messages; use body only when the "why" is non-obvious
- When rewriting history, preserve BOTH the author and the committer date (`GIT_COMMITTER_DATE`) unless told otherwise. `git rebase` resets the committer date to now, which rewrites the timeline without saying so
- Never report a branch as pushed, or a PR as open, without the `git remote -v` / `git status -sb` / `gh pr view` output that proves it
- Before resetting a shared checkout, inspect the reflog and prove HEAD is still yours. Preserve commits and staging added by other sessions
- PR base pointers do not prove Git ancestry. Check ancestry before cascading a stack, and inspect `rebase.updateRefs` so a backup branch is not rewritten with its target
- Leave marking draft PRs ready to the user. Do not edit another author's branch or PR text without authorization: use a separate change or review

## Agent execution

- Execute clear authorized requests directly and confirm concisely
- Use the authorized model, effort, and account without silent fallback. Delegate only when authorized
- Verify process, conversation, and pane together before restarting or resuming an agent. Do not rewrite a live transcript or delete an active session database to clear a lock
- Prompt suggestions are not user instructions. A remembered task does not authorize executing it, and a temporary task constraint does not become a global preference

## Codex memory

- Use Codex native memory for durable preferences, reusable workflows, and non-obvious practical pitfalls. Exclude project designs, settings, exceptions, and task history
- Update memory only on explicit user request through native update notes, and let Codex maintain its generated summaries. Do not maintain or load a separate project memory index or duplicate topic files
- Current user instructions and repository guidance take precedence. An explicit request to update repository documentation belongs in the repository; canonical rules need not be duplicated in memory
- Do not read, search, import, or use Claude memory, its copies, or retired memory archives as a lookup source or fallback. Repository `CLAUDE.md` and `.claude/skills/` remain shared guidance
- Preserve retired native memory detail in Git history, but do not load it into active memory or use it to recreate removed entries

## Python

- Use `uv` for package management
- Target Python >= 3.14
- Prefer `httpx` for HTTP requests
- Prefer `pydantic` for structured data: data models, validation, and (de)serialization, instead of raw dicts or plain dataclasses
- Use PEP 723 inline script metadata with multiline dependencies
- Avoid `python -c '...'` for non-trivial code: write a script (PEP 723) or use a quoted-delimiter heredoc (`uv run python <<'PYTHON'`) to sidestep shell escaping entirely
- Never put backslash-escaped quotes (`\"`) inside an f-string `{}` expression: it is a `SyntaxError` on every Python version; use nested single quotes (`f"{'  '*n}"`)
- Include `main()` function with `if __name__ == "__main__":` guard
- Set `pretty_exceptions_enable=False` on root Typer apps
- Use TOON format for table/list output (not Rich tables or JSON); use `toons` (`from toons import dumps as encode`) for standalone scripts: `toons>=0.7`

## Terraform

- Run `terraform fmt` and `terraform validate` after changes

## Notion

- Prefer `ntn` CLI (notion-cli) for editing Notion pages/DBs in scripts
- Common commands:
  - `ntn pages get <page-id>`: fetch as markdown (properties as frontmatter)
  - `ntn pages create --parent <page|database|data-source:id> --content '...'`
  - `ntn pages update <page-id> --content '...'`: replaces full body
  - `ntn pages trash <page-id>`: move to trash
  - `ntn datasources resolve <database-id>`: get data-source id from DB id
  - `ntn datasources query <data-source-id>`: filter/sort DB rows
  - `ntn api /v1/search -d '{"query":"..."}'`: workspace-wide search (no direct `ntn search` subcommand)
  - `ntn api <path> [-d JSON]`: raw API (e.g. title/property updates via `PATCH /v1/pages/<id>`)
  - `ntn files create < file`: upload

## Search

- Prefer `rg` (ripgrep) over `grep` for searching
- Use the Search tool or Grep tool instead of bash grep commands

## sed

- This environment uses GNU sed (Linux style), not macOS BSD sed
- Use `sed -i 's/old/new/'` (no empty string argument required)
- `\t`, `\n` escape sequences are supported

## Nix

- Keep package lists alphabetically sorted
- Prefer `pkgs.` prefix over `with pkgs;` for small package sets
