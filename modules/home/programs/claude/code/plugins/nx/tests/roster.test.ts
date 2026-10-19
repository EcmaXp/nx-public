import { expect, test } from 'claude-code/testing'

const roster =
  '<system-reminder>\nOther agents active in this session, addressable via SendMessage({to: name, message}): main, hello.\n</system-reminder>'
const hidden =
  '<system-reminder>\nOther agents active in this session, addressable via SendMessage({to: name, message}): (hidden).\n</system-reminder>'

const row = (agentId?: string) =>
  ({
    message: { type: 'user', role: 'user', isMeta: true, content: [{ type: 'text', text: roster }] },
    door: 'note',
    origin: { kind: 'engine' },
    uuid: 'u-1',
    ...(agentId === undefined ? {} : { agentId }),
  }) as never

test('a subagent roster row keeps its prefix and hides the names', async ($, on) => {
  const kept: unknown[] = []
  on('session.append', (_, e, next) => {
    kept.push(e.message.content)
    return next(e)
  })

  await $.session.append(row('ahi-1'))

  expect(kept).toEqual([[{ type: 'text', text: hidden }]])
})

test('the main conversation roster row is left as is', async ($, on) => {
  const kept: unknown[] = []
  on('session.append', (_, e, next) => {
    kept.push(e.message.content)
    return next(e)
  })

  await $.session.append(row())

  expect(kept).toEqual([[{ type: 'text', text: roster }]])
})
