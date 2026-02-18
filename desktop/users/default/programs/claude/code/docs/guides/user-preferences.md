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

## Python

- Use `uv` for package management
- Target Python >= 3.14
- Prefer `httpx` for HTTP requests
- Use PEP 723 inline script metadata with multiline dependencies
- Include `main()` function with `if __name__ == "__main__":` guard
- Set `pretty_exceptions_enable=False` on root Typer apps
- Use TOON format for table/list output (not Rich tables or JSON); use `toon-format` (`toon_format`) for standalone scripts: `toon-format @ git+https://github.com/toon-format/toon-python.git`

## Terraform

- Run `terraform fmt` and `terraform validate` after changes

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
