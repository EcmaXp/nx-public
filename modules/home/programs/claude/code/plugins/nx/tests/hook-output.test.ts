import { expect, test } from 'claude-code/testing'
import { parseHookOutput } from '../hooks/hook-output'

const ok = (stdout: string) => parseHookOutput({ exitCode: 0, stdout, stderr: '' })

test('a rewrite with no permission decision carries no decision of its own', () => {
  const out = ok('{"hookSpecificOutput":{"hookEventName":"PreToolUse","updatedInput":{"command":"git status"}}}')
  expect(out.decision).toEqual({})
  expect(out.updatedInput).toEqual({ command: 'git status' })
})

test('permission decisions map onto allow, ask and deny', () => {
  const decide = (d: string) =>
    ok(`{"hookSpecificOutput":{"permissionDecision":"${d}","permissionDecisionReason":"why"}}`).decision
  expect(decide('allow')).toEqual({ allow: true })
  expect(decide('ask')).toEqual({ ask: 'why' })
  expect(decide('deny')).toEqual({ deny: 'why' })
})

test('exit 2 blocks with stderr, any other failure is ignored as settings hooks are', () => {
  expect(parseHookOutput({ exitCode: 2, stdout: '', stderr: 'no\n' })).toEqual({ decision: { deny: 'no' }, block: 'no' })
  expect(parseHookOutput({ exitCode: 1, stdout: '{"decision":"block"}', stderr: '' })).toEqual({ decision: {} })
  expect(ok('not json')).toEqual({ decision: {} })
})
