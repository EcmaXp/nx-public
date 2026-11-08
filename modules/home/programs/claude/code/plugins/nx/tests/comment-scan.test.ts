import { expect, test } from 'claude-code/testing'
import { parseHookOutput } from '../hooks/hook-output'

test('comment-scan output keeps its context and its user-facing line', () => {
  const out = parseHookOutput({
    exitCode: 0,
    stdout: '{"hookSpecificOutput":{"additionalContext":"Added 1"},"systemMessage":"comment-scan: +1"}',
    stderr: '',
  })
  expect(out.additionalContext).toBe('Added 1')
  expect(out.systemMessage).toBe('comment-scan: +1')
})
