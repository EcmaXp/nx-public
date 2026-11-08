import { expect, test } from 'claude-code/testing'
import { parseEnv } from '../hooks/host'
import { preToolUsePayload } from '../hooks/telemetry'

test('env forwards only the keys sidepulse reads for origin and log path', () => {
  const host = parseEnv(
    ['HOME=/Users/u', 'SIDEPULSE_AGENT_ORIGIN=Claude Code CLI', 'TERM_PROGRAM=ghostty', 'VSCODE_PID=1', 'SECRET=x=y'].join(
      '\n',
    ),
  )
  expect(host.socket).toBe('/Users/u/.local/state/nx/hookd.sock')
  expect(host.sidepulseLog).toBe('/Users/u/.local/state/sidepulse/agent-monitor/claude.jsonl')
  expect(host.forwarded).toEqual({
    SIDEPULSE_AGENT_ORIGIN: 'Claude Code CLI',
    TERM_PROGRAM: 'ghostty',
    VSCODE_PID: '1',
  })
})

test('vibe island bridges are probed in the launcher order unless NX_VIBE_ISLAND_BIN pins one', () => {
  expect(parseEnv('HOME=/Users/u').vibeIslandBridges).toEqual([
    '/Applications/Vibe Island.app/Contents/Helpers/vibe-island-bridge',
    '/Applications/vibe-island.app/Contents/Helpers/vibe-island-bridge',
    '/Users/u/Applications/Vibe Island.app/Contents/Helpers/vibe-island-bridge',
    '/Users/u/.vibe-island/bin/vibe-island-bridge',
  ])
  expect(parseEnv('HOME=/Users/u\nNX_VIBE_ISLAND_BIN=/x/bridge').vibeIslandBridges).toEqual(['/x/bridge'])
})

test('XDG_STATE_HOME moves the sidepulse log but not the daemon socket', () => {
  const host = parseEnv('HOME=/Users/u\nXDG_STATE_HOME=/s')
  expect(host.sidepulseLog).toBe('/s/sidepulse/agent-monitor/claude.jsonl')
  expect(host.socket).toBe('/Users/u/.local/state/nx/hookd.sock')
})

test('a tool.call envelope becomes the PreToolUse stdin a settings hook would read', () => {
  const base = { session_id: 's', transcript_path: '/t', cwd: '/c', permission_mode: 'default' }
  expect(
    preToolUsePayload({ tool: 'Bash', tool_use_id: 'u1', command: 'ls', agentId: 'a1', requestMeta: {} }, base),
  ).toEqual({
    ...base,
    hook_event_name: 'PreToolUse',
    tool_name: 'Bash',
    tool_input: { command: 'ls' },
    tool_use_id: 'u1',
    agent_id: 'a1',
  })
  expect(preToolUsePayload({ tool: 'Read', tool_use_id: 'u2', file_path: '/f' }, base)).not.toHaveProperty('agent_id')
})
