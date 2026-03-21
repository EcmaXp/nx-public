# Git Convention

## Branch Naming

- Use prefix with kebab-case: `prefix/short-description`
- Prefixes: `feature/`, `fix/`, `chore/`, `refactor/`, `docs/`

## Commit Message

- Use Conventional Commits format: `type(scope): description`
- Breaking changes marked with `!` require manual intervention
- When commit message contains `!`, bash history expansion escapes it to `\!`. Use `git commit -F <file>` with a temp file instead of `-m` to avoid this
- Review `git log --oneline -100` before writing commit messages to match existing style
- When a file has 2+ unrelated changes, commit them separately (use `git add -p` to stage individual hunks)

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
