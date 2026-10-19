# Comment Style

- Default to fewer comments in any code I write. Prefer self-explanatory names over explanatory comments
- Drop comments that merely restate what the code does; keep only a *why* that the code cannot carry on its own
- Keep ordinary docstrings to one summary line. Preserve required validation text and machine-read descriptions where they serve a real consumer

A comment is prose inside code, so [writing-style.md](writing-style.md) governs how one reads. The rest of this file governs whether one should exist at all.

## The default is zero

- Try the cheaper fix first: a better name. Rename the variable, split the function, or lift the condition into a named boolean. A name cannot drift out of sync with the code; a comment can
- If a comment survives that test, keep it to one line
- Never narrate a change in a comment ("changed from 15s", "was previously per-account")

## Where intent goes

Intent has a home, and it is usually not the source file. In order:

1. **The PR description.** This is the default. Reasoning, alternatives considered, and measurements belong here, where they are searchable and cannot rot against the code
1. **A PR inline comment**, or a comment added to the PR later, when the point is about one specific line and the reader needs it at review time
1. **A one-line code comment**, only when a reader will still need the fact long after the PR is forgotten and no name or test can carry it

## What stays

- A *why* the code cannot carry: a constraint from outside the file, a bug the shape works around, a limit that someone would otherwise "fix"
- Tool directives: `# noqa`, `# type: ignore`, `// eslint-disable`
- A `ponytail:` marker naming a deliberate shortcut and its upgrade path

## Two pairs

Delete, because the code already says it:

```python
# increment the counter
counter += 1
```

Keep, because nothing in the file could tell you this:

```python
# The API rejects a page size over 200, and does not document the limit.
page_size = 200
```

Delete, because a table entry is not where reasoning belongs:

```rust
// css uses block comments, since --gap is a custom property
("css", BLOCK),
```

Keep the same fact as a test name instead, where it runs:

```rust
fn css_counts_blocks_but_not_custom_properties() { ... }
```

## A name that runs beats a comment

Prefer a named test over a comment above the line it protects. The test cannot fall out of sync, and it fails when the reasoning breaks. `nx-comment-scan` is written this way: zero comments, and every non-obvious choice pinned by a test whose name states it.

## The hook

A `PostToolUse` hook counts the comment lines each edit adds, reports the count to both you and me, and orders every added line deleted, sparing only machine-read directives.

A shell edit carries no before-state, so there the hook parses the command and counts the comment lines in what it writes: a heredoc body, a `sed` script. It reports those as `~N`. A throwaway script inside a heredoc gets the same treatment, and the same rules.
