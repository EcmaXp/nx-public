import type { Engine, Register } from 'claude-code'
import { passOn, runHook } from './hook-output'

async function onPostToolUse<E extends { tool_name: string }, R>($: Engine, e: E, next: (e: E) => Promise<R>) {
  const [scan, below] = await Promise.all([runHook($, 'nx-comment-scan', [], e), next(e)])
  const result = below as R & { additionalContext?: string[]; block?: string }
  if (scan.systemMessage) $.ui.log(scan.systemMessage)
  return {
    ...result,
    ...(scan.additionalContext
      ? { additionalContext: [...(result.additionalContext ?? []), scan.additionalContext] }
      : {}),
    ...(scan.block && !result.block ? { block: scan.block } : {}),
  }
}

export const register: Register = on => {
  on('classic.PostToolUse', { tool_name: /^(Write|Edit|MultiEdit|Bash)$/ }, onPostToolUse).catch(passOn)
}
