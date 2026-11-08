import type { Engine, Register } from 'claude-code'
import { binaryOf, firstFile, hostOf } from './host'

const DAEMON_RETRY_MS = 30_000

const SIDEPULSE_EVENTS = new Set([
  'SessionStart',
  'SessionEnd',
  'UserPromptSubmit',
  'PreToolUse',
  'PostToolUse',
  'PostToolUseFailure',
  'PermissionRequest',
  'Notification',
  'PreCompact',
  'PostCompact',
  'Stop',
  'SubagentStop',
])
const VIBE_ISLAND_EVENTS = new Set([
  'SessionStart',
  'SessionEnd',
  'UserPromptSubmit',
  'PreToolUse',
  'PostToolUse',
  'Notification',
  'PreCompact',
  'Stop',
  'StopFailure',
  'SubagentStart',
  'SubagentStop',
])

export type Base = { session_id?: string; transcript_path?: string; cwd?: string; permission_mode?: string }
type Payload = Base & { hook_event_name: string }

let vibeIslandBridge: Promise<string | undefined> | undefined
let base: Base = {}
let daemonDownUntil = 0
let sidepulseQueue: Promise<void> = Promise.resolve()
let vibeIslandQueue: Promise<void> = Promise.resolve()

export const sessionBase = () => base

export const preToolUsePayload = (e: Record<string, unknown>, base: Base): Payload => {
  const { tool, tool_use_id, agentId, requestMeta, ...tool_input } = e
  return {
    ...base,
    hook_event_name: 'PreToolUse',
    tool_name: tool,
    tool_input,
    tool_use_id,
    ...(agentId === undefined ? {} : { agent_id: agentId }),
  }
}

async function sidepulse($: Engine, payload: Payload) {
  const { socket, sidepulseLog, forwarded } = await hostOf($)
  const body = JSON.stringify(payload)
  if (Date.now() >= daemonDownUntil) {
    const sent = await $.http
      .fetch('http://hookd/sidepulse/claude', {
        method: 'POST',
        socketPath: socket,
        headers: { 'content-type': 'application/json', 'x-hook-env': JSON.stringify(forwarded) },
        body,
      })
      .then(r => r.ok)
      .catch(() => false)
    if (sent) return
    daemonDownUntil = Date.now() + DAEMON_RETRY_MS
  }
  const cli = await binaryOf($, 'sidepulse')
  if (cli) await $.process.run([cli, 'hook-log', '--provider', 'claude', '--log', sidepulseLog], { stdin: body })
}

async function vibeIsland($: Engine, payload: Payload) {
  const bridge = await (vibeIslandBridge ??= hostOf($).then(h => firstFile($, h.vibeIslandBridges)))
  if (bridge) await $.process.run([bridge, '--source', 'claude'], { stdin: JSON.stringify(payload) })
}

function reportInOrder($: Engine, payload: Payload) {
  const event = payload.hook_event_name
  if (SIDEPULSE_EVENTS.has(event)) sidepulseQueue = sidepulseQueue.then(() => sidepulse($, payload)).catch(() => {})
  if (VIBE_ISLAND_EVENTS.has(event))
    vibeIslandQueue = vibeIslandQueue.then(() => vibeIsland($, payload)).catch(() => {})
}

async function onClassic<E extends Payload, R>($: Engine, e: E, next: (e: E) => R) {
  const { session_id, transcript_path, cwd, permission_mode } = e
  base = { session_id, transcript_path, cwd, permission_mode }
  reportInOrder($, e)
  if (e.hook_event_name === 'SessionEnd') await Promise.all([sidepulseQueue, vibeIslandQueue])
  return next(e)
}

async function onToolCall<E, R>($: Engine, e: E, next: (e: E) => R) {
  const session_id = base.session_id ?? (await $.session.id())
  reportInOrder($, preToolUsePayload(e as Record<string, unknown>, { ...base, session_id }))
  return next(e)
}

function passOn<E, R>(_: Engine, e: E, next: (e: E) => R) {
  return next(e)
}

export const register: Register = on => {
  on('classic.SessionStart', onClassic).catch(passOn)
  on('classic.SessionEnd', onClassic).catch(passOn)
  on('classic.UserPromptSubmit', onClassic).catch(passOn)
  on('classic.PostToolUse', onClassic).catch(passOn)
  on('classic.PostToolUseFailure', onClassic).catch(passOn)
  on('classic.PermissionRequest', onClassic).catch(passOn)
  on('classic.Notification', onClassic).catch(passOn)
  on('classic.PreCompact', onClassic).catch(passOn)
  on('classic.PostCompact', onClassic).catch(passOn)
  on('classic.Stop', onClassic).catch(passOn)
  on('classic.StopFailure', onClassic).catch(passOn)
  on('classic.SubagentStart', onClassic).catch(passOn)
  on('classic.SubagentStop', onClassic).catch(passOn)
  on('tool.call', onToolCall).catch(passOn)
}
