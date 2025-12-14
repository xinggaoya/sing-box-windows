import { invokeWithAppContext } from './invoke-client'

export interface SudoPasswordStatus {
  supported: boolean
  has_saved: boolean
}

export const sudoService = {
  getStatus() {
    return invokeWithAppContext<SudoPasswordStatus>('sudo_password_status', undefined, {
      skipDataRestore: true,
    })
  },

  setPassword(password: string) {
    return invokeWithAppContext<void>(
      'sudo_set_password',
      { password },
      { skipDataRestore: true }
    )
  },

  clearPassword() {
    return invokeWithAppContext<void>('sudo_clear_password', undefined, { skipDataRestore: true })
  },
}

