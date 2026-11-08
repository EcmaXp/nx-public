import { expect, test } from 'claude-code/testing'
import { hideRosterNames } from '../hooks/roster'

const roster =
  'Other agents active in this session, addressable via SendMessage({to: name, message}): main, hello.'
const hidden = 'Other agents active in this session, addressable via SendMessage({to: name, message}): (hidden).'

test('the roster line keeps its prefix and hides the names', () => {
  expect(hideRosterNames(roster)).toEqual(hidden)
})

test('text without a roster line is left as is', () => {
  expect(hideRosterNames('nothing to hide')).toEqual('nothing to hide')
})
