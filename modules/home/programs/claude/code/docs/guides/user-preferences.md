# User Preferences

## Environment

- macOS with Nix package manager
- CLI tools are GNU versions (not BSD) from `/run/current-system/sw/bin/`

## Claude Code Sandbox

- `Operation not permitted` errors during git worktree operations indicate sandbox restrictions
- Disable sandbox for worktree creation: use `dangerouslyDisableSandbox: true`
- Affected operations: `git worktree add`, large repo checkouts, file creation in new directories

## Git

- Run `git commit` separately (no `&&` or `;` chaining)
- Use conventional commits with scope when applicable (e.g., `feat(api):`, `fix(auth):`, `docs(readme):`)
- Review `git log` before committing to match repository's commit style
- When splitting into multiple commits: review all changes first (`git diff`), then stage specific files per commit
- Prefer single-line commit messages; use body only when the "why" is non-obvious

## Writing

- Avoid the em-dash (`—`); it reads as machine-generated. Use what humans here actually reach for instead:
  - Colon (`:`) to introduce an explanation, definition, or list
  - Middle dot (`·`) to join paired or related terms (e.g., `참조·순환`, `의도·문제`)
  - Arrow (`→`) for sequence or flow (e.g., `입력 → 처리 → 출력`)
  - Parentheses for an aside or clarification
  - Or split into two sentences, or join with a comma
- Avoid the en-dash (`–`): plain hyphen for ranges, or `~` in Korean (`2024~2025`)
- Use straight quotes (`"` `'`), not curly quotes (`“” ‘’`)
- Use three dots (`...`), not the ellipsis character (`…`)

## Code Comments

- Default to fewer comments in any code I write. Prefer self-explanatory names over explanatory comments
- Drop comments that merely restate what the code does; keep only a *why* that the code cannot carry on its own
- Keep docstrings and machine-read text (e.g. `error_message`, schema descriptions) where they serve a real consumer

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
- Use TOON format for table/list output (not Rich tables or JSON); use `toon-format` (`toon_format`) for standalone scripts: `toon-format @ git+https://github.com/toon-format/toon-python.git`

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
