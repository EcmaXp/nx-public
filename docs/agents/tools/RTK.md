# RTK - Rust Token Killer

RTK is a token-optimized CLI proxy for development commands.

## Usage

Always prefix shell commands with `rtk`. In a command chain, prefix each
segment.

```bash
rtk git status
rtk git add . && rtk git commit -m "message"
rtk cargo test
rtk npm run build
```

Use a raw command only when the RTK-filtered output does not contain enough
detail for debugging. Use `rtk proxy <cmd>` when you want unfiltered output
while keeping RTK usage tracking.

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
