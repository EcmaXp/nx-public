---
name: git-convention
description: Git branch naming, commit message format, and conventional commits conventions. Load when using git, creating commits, branches, or pull requests.
---

# Git Convention

When a PR description or review comment is written in Korean, follow `~/.agents/docs/guides/korean-style.md`. Commit messages and branch names stay English.

## Branch Naming

- Use prefix with kebab-case: `prefix/short-description`
- Prefixes: `feature/`, `fix/`, `chore/`, `refactor/`, `docs/`

## Commit Message

- Use Conventional Commits format: `type(scope): description`
- Breaking changes marked with `!` require manual intervention
- When commit message contains `!`, bash history expansion escapes it to `\!`. Use `git commit -F <file>` with a temp file instead of `-m` to avoid this
- Review `git log --oneline -100` before writing commit messages to match existing style
- When a file has 2+ unrelated changes, commit them separately (use `git add -p` to stage individual hunks)
- Stage and commit as one atomic command: `git add <files> && git commit --only <dir> -m "..."` with `--only` scoped to the staged paths' parent folder, never a bare `git commit` (hunk-level splits above are the exception: `git add -p`, then a bare commit)
- Brace-expand sibling files sharing a prefix: `git add path/{a,b,c}.py` (needs at least one comma, `{a}` stays literal; unquoted)
- The Bash tool runs zsh, which does **not** word-split an unquoted `$VAR`: `git reset -- $PATHS` passes one joined argument and silently matches nothing. Brace-expand instead
- Join a command to the one it depends on with `&&`, never `;`, and never silence it with `>/dev/null 2>&1`. A failed `git stash push` followed by `; git stash pop` popped an unrelated stash into a conflicted index, and the redirect hid the first failure
- Declare a temp variable for repeated path prefixes and brace-expand siblings: `D=<dir>; git add $D/{a,b}.py && git commit --only $D -m "..."` (for an out-of-cwd repo, add `gw $D`, next bullet)
- For a repo outside the cwd, never `git -C` (or `cd`): `D=<shared-prefix-dir>; gw $D` — the `gw` helper (shell-env.sh, loaded in every Bash call) discovers the enclosing repo and exports `GIT_WORK_TREE`/`GIT_DIR`; keep referencing paths as `$D/...`. Fallback without the helper: `export GIT_WORK_TREE=<abs-repo>; export GIT_DIR=$GIT_WORK_TREE/.git` (two statements: a single export expands `$GIT_WORK_TREE` before the assignment lands)
- `gw` is only for a repo the cwd is **not** in, and it refuses when the cwd already sits under the target. Inside your own repo the pin buys nothing, and git still resolves a pathspec against the cwd, so a repo-root-relative path quietly means `<cwd>/<path>`. Pass `$D/...`, or prefix `:/` to anchor at the root (`git stash push -- :/projects/x/y.py`)
- Never set `GIT_DIR` alone: without `GIT_WORK_TREE` the work tree silently falls back to the cwd. The export redirects every later git/gh call in the same Bash call (`gw -u` drops it; `gw` refuses to repin a different repo while set); env resets between calls, so re-run per call

## Commit Types

- `feat`: New features or enhancements (most common)
- `fix`: Bug fixes or issue corrections
- `chore`: Maintenance tasks, dependency updates, code style/formatting changes
- `refactor`: Code restructuring without changing behavior
- `docs`: Documentation updates

## Breaking Changes

- Indicated by `!` before colon: `feat(pkgs)!: remove package`
- Always review impact before merging
- May require manual intervention during updates

## Commit Description

- Use imperative, present tense verbs
- Start with lowercase
- Keep commit title only, messages concise (\<= 64 characters)
- Action verbs to use:
  - Adding: `add`, `create`, `implement`, `introduce`
  - Changing: `update`, `improve`, `enhance`, `refine`, `modify`
  - Removing: `remove`, `delete`, `clean`, `eliminate`
  - Fixing: `fix`, `repair`, `resolve`, `address`
  - Others: `use`, `map`, `enable`, `disable`, `migrate`
