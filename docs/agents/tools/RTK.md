# RTK - Rust Token Killer

RTK is a token-optimized CLI proxy for development commands.

## Usage

A Claude Code PreToolUse hook (`rtk hook claude`) rewrites supported commands
automatically. Do not type `rtk` in front of commands yourself: a manual
`rtk grep` or `rtk diff` bypasses the exclusions below.

`modules/home/programs/claude/code/rtk.toml` excludes `diff`, `git diff`,
`git show`, `grep`, and `rg` from the hook, because rtk changes their meaning:

- `git diff` and `git show` come out compacted and are no longer valid patches
- `rtk diff` reports "identical" for line-ending or final-newline differences, and always exits 0
- `rtk grep -v` ignores `-v`

Run those commands as they are. Use `rtk proxy <cmd>` only when you need raw
output from a command the hook does rewrite.

## Meta Commands (always use rtk directly)

```bash
rtk gain              # Show token savings analytics
rtk gain --history    # Show command usage history with savings
rtk discover          # Analyze Claude Code history for missed opportunities
rtk proxy <cmd>       # Execute raw command without filtering (for debugging)
```

## Installation Verification

```bash
rtk --version         # Should show: rtk X.Y.Z
rtk gain              # Should work (not "command not found")
rtk proxy which rtk   # Verify correct binary
```

**Name collision**: If `rtk gain` fails, you may have
reachingforthejack/rtk (Rust Type Kit) installed instead.
