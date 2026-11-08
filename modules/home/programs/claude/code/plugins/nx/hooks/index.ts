import type { Register } from 'claude-code'
import { register as commentScan } from './comment-scan'
import { register as roster } from './roster'
import { register as telemetry } from './telemetry'

export const register: Register = (on, options) => {
  roster(on, options)
  telemetry(on, options)
  commentScan(on, options)
}
