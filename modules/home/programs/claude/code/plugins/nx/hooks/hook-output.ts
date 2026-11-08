import type { Engine } from 'claude-code'
import { binaryOf } from './host'

const HOOK_TIMEOUT_MS = 5_000

type Decision = { allow: true } | { ask: string } | { deny: string } | Record<string, never>
type HookOutput = {
  decision: Decision
  updatedInput?: Record<string, unknown>
  additionalContext?: string
  systemMessage?: string
  block?: string
}

export const parseHookOutput = ({
  exitCode,
  stdout,
  stderr,
}: {
  exitCode: number
  stdout: string
  stderr: string
}): HookOutput => {
  if (exitCode === 2) return { decision: { deny: stderr.trim() }, block: stderr.trim() }
  if (exitCode !== 0 || !stdout.trim()) return { decision: {} }
  let json: Record<string, any>
  try {
    json = JSON.parse(stdout)
  } catch {
    return { decision: {} }
  }
  const specific = json.hookSpecificOutput ?? {}
  const reason = String(specific.permissionDecisionReason ?? '')
  const decision: Decision =
    specific.permissionDecision === 'deny'
      ? { deny: reason }
      : specific.permissionDecision === 'ask'
        ? { ask: reason }
        : specific.permissionDecision === 'allow'
          ? { allow: true }
          : {}
  return {
    decision,
    updatedInput: specific.updatedInput,
    additionalContext: specific.additionalContext,
    systemMessage: json.systemMessage,
    block: json.decision === 'block' ? String(json.reason ?? '') : undefined,
  }
}

export async function runHook($: Engine, name: string, args: string[], payload: object) {
  const bin = await binaryOf($, name)
  if (!bin) return parseHookOutput({ exitCode: 1, stdout: '', stderr: '' })
  return $.process
    .run([bin, ...args], { stdin: JSON.stringify(payload), timeoutMs: HOOK_TIMEOUT_MS })
    .then(parseHookOutput, () => parseHookOutput({ exitCode: 1, stdout: '', stderr: '' }))
}

export function passOn<E, R>(_: Engine, e: E, next: (e: E) => R) {
  return next(e)
}
