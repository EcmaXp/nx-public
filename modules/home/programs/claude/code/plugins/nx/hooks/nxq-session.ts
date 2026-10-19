import type { Register } from 'claude-code'

const NXQ_COMMAND = /(^|[^\w-])(nxq|nx-quiet)([^\w-]|$)/

export const withChildSession = (command: string, sessionId: string, agentId: string) => {
  if (!NXQ_COMMAND.test(command)) return command
  const child = `claude:${sessionId}:${agentId}`
  return `export NXQ_SESSION='${child}' NXQ_OWNER='${child}'; ${command}`
}

export const register: Register = on => {
  on('tool.call', { tool: 'Bash' }, async ($, e, next) => {
    if (e.agentId === undefined) return next(e)
    const command = withChildSession(e.command, await $.session.id(), e.agentId)
    return next(command === e.command ? e : { ...e, command })
  }).catch(($, e, next) => next(e))
}
