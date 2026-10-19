import { expect, test } from 'claude-code/testing'
import { withChildSession } from '../hooks/nxq-session'

test('only commands that run nxq get the child session', () => {
  expect(withChildSession('nxq read a.py', 's', 'a1')).toBe(
    "export NXQ_SESSION='claude:s:a1' NXQ_OWNER='claude:s:a1'; nxq read a.py",
  )
  expect(withChildSession('cd x && nx-quiet run ls', 's', 'a1')).toContain("NXQ_SESSION='claude:s:a1'")
  expect(withChildSession('git status', 's', 'a1')).toBe('git status')
  expect(withChildSession('ls nxq-notes', 's', 'a1')).toBe('ls nxq-notes')
})

test('a main-thread Bash call passes through unchanged', async ($, on) => {
  const ran: string[] = []
  on('tool.call', { tool: 'Bash' }, (_, e) => {
    ran.push(e.command)
    return { result: {} } as never
  })

  await $.tool.call({ tool: 'Bash', command: 'nxq read a.py' } as never)

  expect(ran).toEqual(['nxq read a.py'])
})
