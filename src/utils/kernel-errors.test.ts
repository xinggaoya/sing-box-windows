import { describe, expect, it, vi } from 'vitest'
import {
  describeKernelFailure,
  extractKernelErrorMessage,
  parseSudoCode,
  resolveKernelFailureText,
} from '@/utils/kernel-errors'

/**
 * 这些函数此前在 App.vue / HomeView.vue / TrayStore.ts 各写了一份逐字相同的实现，
 * 只要后端改了错误码前缀就会静默不一致（表现为「某些入口不弹密码框」）。
 * 因此测试重点覆盖「各种错误形态都能正确识别」。
 */

describe('extractKernelErrorMessage', () => {
  it('原样返回裸字符串', () => {
    expect(extractKernelErrorMessage('boom')).toBe('boom')
  })

  it('从 Error 实例取 message', () => {
    expect(extractKernelErrorMessage(new Error('kernel died'))).toBe('kernel died')
  })

  it('从 { message } 对象取值', () => {
    expect(extractKernelErrorMessage({ message: 'from object' })).toBe('from object')
  })

  it('从 { error } 对象取值', () => {
    expect(extractKernelErrorMessage({ error: 'from error field' })).toBe('from error field')
  })

  it('把非字符串 error 字段转成字符串', () => {
    expect(extractKernelErrorMessage({ error: 500 })).toBe('500')
  })

  it('null / undefined 归一化为空串', () => {
    expect(extractKernelErrorMessage(null)).toBe('')
    expect(extractKernelErrorMessage(undefined)).toBe('')
  })

  it('message 非字符串时继续尝试 error 字段', () => {
    expect(extractKernelErrorMessage({ message: 123, error: 'fallback' })).toBe('fallback')
  })

  it('数字等其他原始值走 String()', () => {
    expect(extractKernelErrorMessage(404)).toBe('404')
  })
})

describe('parseSudoCode', () => {
  it('识别 SUDO_PASSWORD_REQUIRED', () => {
    expect(parseSudoCode('SUDO_PASSWORD_REQUIRED')).toBe('required')
  })

  it('识别 SUDO_PASSWORD_INVALID', () => {
    expect(parseSudoCode('SUDO_PASSWORD_INVALID')).toBe('invalid')
  })

  it('能从嵌套在后端错误消息中的代码识别', () => {
    expect(parseSudoCode(new Error('启动内核失败: SUDO_PASSWORD_INVALID'))).toBe('invalid')
    expect(parseSudoCode({ message: 'SUDO_PASSWORD_REQUIRED' })).toBe('required')
  })

  it('与 sudo 无关的错误返回 null', () => {
    expect(parseSudoCode('connection refused')).toBeNull()
    expect(parseSudoCode(null)).toBeNull()
  })

  it('大小写敏感（后端约定为大写常量）', () => {
    expect(parseSudoCode('sudo_password_required')).toBeNull()
  })
})

describe('resolveKernelFailureText', () => {
  it('诊断摘要优先级最高', () => {
    expect(resolveKernelFailureText('诊断摘要', '上次错误', '兜底')).toBe('诊断摘要')
  })

  it('无诊断摘要时用 lastError', () => {
    expect(resolveKernelFailureText('', '上次错误', '兜底')).toBe('上次错误')
    expect(resolveKernelFailureText(null, '上次错误', '兜底')).toBe('上次错误')
  })

  it('两者都缺时用兜底', () => {
    expect(resolveKernelFailureText('', '', '兜底')).toBe('兜底')
    expect(resolveKernelFailureText(null, undefined, '兜底')).toBe('兜底')
  })
})

describe('describeKernelFailure', () => {
  it('优先使用本次操作真实抛出的消息（比历史 lastError 更贴近当前原因）', () => {
    const failure = describeKernelFailure('SUDO_PASSWORD_REQUIRED', {
      diagnosisSummary: '旧的诊断',
      lastError: '更旧的错误',
      fallback: '兜底',
    })
    expect(failure.text).toBe('SUDO_PASSWORD_REQUIRED')
    expect(failure.sudoCode).toBe('required')
    expect(failure.needsSudoRetry).toBe(true)
  })

  it('本次没有可读消息时回退到诊断摘要', () => {
    const failure = describeKernelFailure(null, {
      diagnosisSummary: '内核启动失败：端口被占用',
      lastError: '别的错误',
      fallback: '兜底',
    })
    expect(failure.text).toBe('内核启动失败：端口被占用')
  })

  it('与 sudo 无关的失败不需要重试', () => {
    const failure = describeKernelFailure(new Error('port already in use'), {
      fallback: '内核重启失败',
    })
    expect(failure.needsSudoRetry).toBe(false)
    expect(failure.sudoCode).toBeNull()
    expect(failure.text).toBe('port already in use')
  })

  it('三级回退全部落空时返回非空兜底文案', () => {
    const failure = describeKernelFailure(undefined, {
      diagnosisSummary: '',
      lastError: '',
      fallback: '内核重启失败',
    })
    expect(failure.text).toBe('内核重启失败')
    expect(failure.text.length).toBeGreaterThan(0)
  })

  it('invalid 与 required 都标记为需要重试', () => {
    for (const code of ['SUDO_PASSWORD_REQUIRED', 'SUDO_PASSWORD_INVALID']) {
      expect(describeKernelFailure(code, { fallback: 'x' }).needsSudoRetry).toBe(true)
    }
  })
})

describe('错误消息不依赖 console 输出', () => {
  it('函数是纯函数（便于在任意层复用）', () => {
    const spy = vi.spyOn(console, 'error')
    parseSudoCode('SUDO_PASSWORD_REQUIRED')
    expect(spy).not.toHaveBeenCalled()
  })
})
