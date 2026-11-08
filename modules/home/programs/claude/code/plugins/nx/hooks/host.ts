import type { Engine } from 'claude-code'

const FORWARDED_ENV = /^(SIDEPULSE_AGENT_ORIGIN(_KIND)?|TERM_PROGRAM|XDG_STATE_HOME|VSCODE_\w+)$/
const VIBE_ISLAND_HELPER = 'Contents/Helpers/vibe-island-bridge'

export type Host = {
  socket: string
  sidepulseLog: string
  forwarded: Record<string, string>
  vibeIslandBridges: string[]
  path: string[]
}

let host: Promise<Host> | undefined
const binaries = new Map<string, Promise<string | undefined>>()

export const parseEnv = (text: string): Host => {
  const env: Record<string, string> = {}
  for (const line of text.split('\n')) {
    const eq = line.indexOf('=')
    if (eq > 0) env[line.slice(0, eq)] = line.slice(eq + 1)
  }
  const state = env.XDG_STATE_HOME || `${env.HOME}/.local/state`
  return {
    socket: `${env.HOME}/.local/state/nx/hookd.sock`,
    sidepulseLog: `${state}/sidepulse/agent-monitor/claude.jsonl`,
    forwarded: Object.fromEntries(Object.entries(env).filter(([key]) => FORWARDED_ENV.test(key))),
    vibeIslandBridges: env.NX_VIBE_ISLAND_BIN
      ? [env.NX_VIBE_ISLAND_BIN]
      : [
          `/Applications/Vibe Island.app/${VIBE_ISLAND_HELPER}`,
          `/Applications/vibe-island.app/${VIBE_ISLAND_HELPER}`,
          `${env.HOME}/Applications/Vibe Island.app/${VIBE_ISLAND_HELPER}`,
          `${env.HOME}/.vibe-island/bin/vibe-island-bridge`,
        ],
    path: (env.PATH ?? '').split(':').filter(Boolean),
  }
}

export function hostOf($: Engine) {
  return (host ??= $.process.run(['/usr/bin/env']).then(r => parseEnv(r.stdout)))
}

export async function firstFile($: Engine, paths: readonly string[]) {
  for (const path of paths) {
    const isFile = await $.fs.stat(path).then(
      s => s.kind === 'file',
      () => false,
    )
    if (isFile) return path
  }
  return undefined
}

async function findBinary($: Engine, name: string) {
  return firstFile(
    $,
    (await hostOf($)).path.map(dir => `${dir}/${name}`),
  )
}

export function binaryOf($: Engine, name: string) {
  let found = binaries.get(name)
  if (!found) binaries.set(name, (found = findBinary($, name)))
  return found
}
