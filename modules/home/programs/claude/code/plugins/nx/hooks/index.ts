import type { Register } from 'claude-code'
import { register as nxqSession } from './nxq-session'
import { register as roster } from './roster'

export const register: Register = (on, options) => {
  roster(on, options)
  nxqSession(on, options)
}
