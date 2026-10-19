import type { Register } from 'claude-code'

const ROSTER_NAMES = /(Other agents active in this session, addressable via [^\n]*?\): )[^\n]*/

export const hideRosterNames = (text: string) => text.replace(ROSTER_NAMES, '$1(hidden).')

export const register: Register = on => {
  on('session.append', ($, e, next) => {
    if (e.agentId === undefined || e.message.isMeta !== true) return next(e)

    const content = e.message.content
    const rewritten =
      typeof content === 'string'
        ? hideRosterNames(content)
        : content.map(block => (block.type === 'text' ? { ...block, text: hideRosterNames(block.text) } : block))
    return next({ ...e, message: { ...e.message, content: rewritten } })
  }).catch(($, e, next) => next(e))
}
