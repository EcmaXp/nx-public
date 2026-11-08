import type { Register } from 'claude-code'

const ROSTER_NAMES = /(Other agents active in this session, addressable via [^\n]*?\): )[^\n]*/

export const hideRosterNames = (text: string) => text.replace(ROSTER_NAMES, '$1(hidden).')

export const register: Register = on => {
  on('prompt.attachment', ($, e, next) => {
    if (e.agentId === undefined || e.origin.kind !== 'engine') return next(e)
    const text = hideRosterNames(e.text)
    return next(text === e.text ? e : { ...e, text })
  }).catch(($, e, next) => next(e))
}
