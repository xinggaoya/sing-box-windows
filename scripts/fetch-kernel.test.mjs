import test from 'node:test'
import assert from 'node:assert/strict'
import os from 'node:os'
import path from 'node:path'
import {
  buildDownloadUrls,
  buildFilename,
  buildGithubHeaders,
  buildTarAttempts,
  normalizeVersionTag,
  parseArgs,
  parseVersionFromReleaseHtml,
  parseVersionFromReleaseUrl,
  resolveRequestedTargets,
  windowsSystemTar
} from './fetch-kernel.mjs'

test('parseArgs 支持等号和值分离两种形式', () => {
  const args = parseArgs([
    '--platform',
    'windows',
    '--arch=amd64',
    '--skip-existing',
    '--out',
    'tmp/kernel'
  ])

  assert.deepEqual(args, {
    platform: 'windows',
    arch: 'amd64',
    'skip-existing': true,
    out: 'tmp/kernel'
  })
})

test('resolveRequestedTargets 支持主机目标和全量目标', () => {
  const hostTargets = resolveRequestedTargets({}, { platform: 'win32', arch: 'x64' })
  assert.equal(hostTargets.length, 1)
  assert.equal(hostTargets[0].platform, 'windows')
  assert.equal(hostTargets[0].arch, 'amd64')

  const allTargets = resolveRequestedTargets({ all: true }, { platform: 'linux', arch: 'x64' })
  // 目标矩阵共 5 个：windows/amd64、windows/arm64、linux/amd64、macos/arm64、macos/amd64
  // （新增 windows/arm64 时忘了同步这条断言，且该文件此前没接入 CI，故一直没人发现）
  assert.equal(allTargets.length, 5)
})

test('buildFilename 根据平台生成正确文件名', () => {
  assert.equal(buildFilename('windows', 'amd64', '1.12.0'), 'sing-box-1.12.0-windows-amd64.zip')
  assert.equal(buildFilename('macos', 'arm64', '1.12.0'), 'sing-box-1.12.0-darwin-arm64.tar.gz')
  assert.equal(buildFilename('linux', 'amd64', '1.12.0'), 'sing-box-1.12.0-linux-amd64.tar.gz')
})

test('buildDownloadUrls 返回镜像与原始下载地址', () => {
  const urls = buildDownloadUrls('1.12.0', 'sing-box-1.12.0-windows-amd64.zip')

  assert.equal(urls.length, 7)
  assert.match(urls[0], /gh-proxy/)
  assert.equal(
    urls.at(-1),
    'https://github.com/SagerNet/sing-box/releases/download/v1.12.0/sing-box-1.12.0-windows-amd64.zip'
  )
})

test('版本辅助函数可以解析 tag、URL 和 HTML', () => {
  assert.equal(normalizeVersionTag(' v1.12.0 '), '1.12.0')
  assert.equal(normalizeVersionTag('1.12.0'), '1.12.0')
  assert.equal(normalizeVersionTag(''), null)

  assert.equal(
    parseVersionFromReleaseUrl('https://github.com/SagerNet/sing-box/releases/tag/v1.12.0'),
    '1.12.0'
  )
  assert.equal(parseVersionFromReleaseUrl('https://example.com/releases/latest'), null)

  assert.equal(
    parseVersionFromReleaseHtml('<a href="/SagerNet/sing-box/releases/tag/v1.12.1">release</a>'),
    '1.12.1'
  )
  assert.equal(parseVersionFromReleaseHtml('<html></html>'), null)
})

test('buildGithubHeaders 在存在 token 时附带认证头', () => {
  const original = process.env.SING_BOX_GITHUB_TOKEN
  process.env.SING_BOX_GITHUB_TOKEN = 'secret-token'

  try {
    const headers = buildGithubHeaders(true)
    assert.equal(headers.Authorization, 'Bearer secret-token')
    assert.equal(headers['X-GitHub-Api-Version'], '2022-11-28')
    assert.equal(headers.Accept, 'application/vnd.github+json')
  } finally {
    if (original === undefined) {
      delete process.env.SING_BOX_GITHUB_TOKEN
    } else {
      process.env.SING_BOX_GITHUB_TOKEN = original
    }
  }
})

test('buildTarAttempts 兜底走相对路径，绕开 MSYS 盘符冒号转换', () => {
  // Windows 上 Git Bash 自带的 GNU tar 会被 MSYS runtime 做参数转换：
  // "C:/..." 的冒号被转义成 "C\:/..."，tar 收到一个不存在的路径。
  // 兜底方案把工作目录设成归档所在目录、只传相对路径，天然不含盘符冒号。
  const workDir = path.join(os.tmpdir(), 'sing-box-abc')
  const archivePath = path.join(workDir, 'sing-box-1.14.4-linux-amd64.tar.gz')
  const outputDir = path.join(workDir, 'extract')

  const attempts = buildTarAttempts(['-xzf'], archivePath, outputDir)
  const fallback = attempts[attempts.length - 1]

  assert.equal(fallback.command, 'tar')
  assert.equal(fallback.cwd, workDir, '兜底调用应把工作目录设为归档所在目录')
  assert.ok(
    !fallback.args.some((arg) => path.isAbsolute(arg)),
    `兜底调用的参数不应含绝对路径: ${fallback.args.join(' ')}`
  )
  assert.ok(
    fallback.args.includes('sing-box-1.14.4-linux-amd64.tar.gz'),
    `兜底调用应传归档文件名: ${fallback.args.join(' ')}`
  )
  assert.equal(fallback.args[fallback.args.length - 1], 'extract')
})

test('buildTarAttempts 在 Windows 上优先使用系统自带 bsdtar', () => {
  // 系统自带的 tar.exe 是原生 Windows 程序，不经过 MSYS 路径转换，
  // 原生吃盘符路径（虽然不认 --force-local，但根本不需要）。
  const archivePath = path.join(os.tmpdir(), 'sb.tar.gz')
  const outputDir = path.join(os.tmpdir(), 'out')

  const attempts = buildTarAttempts(['-xzf'], archivePath, outputDir)

  // Windows 自带 bsdtar 不一定存在（Windows 10 1803 之前没有 tar.exe），
  // 没有时直接走通用兜底候选。
  if (process.platform !== 'win32' || windowsSystemTar() === null) {
    assert.equal(attempts[0].command, 'tar')
    return
  }

  assert.match(attempts[0].command, /tar\.exe$/i)
  assert.ok(
    attempts[0].args.includes(archivePath),
    `bsdtar 候选应直接使用原生归档路径: ${attempts[0].args.join(' ')}`
  )
})
