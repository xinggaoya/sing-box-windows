import test from 'node:test'
import assert from 'node:assert/strict'
import {
  buildDownloadUrls,
  buildFilename,
  buildGithubHeaders,
  normalizeVersionTag,
  parseArgs,
  parseVersionFromReleaseHtml,
  parseVersionFromReleaseUrl,
  resolveRequestedTargets
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
  assert.equal(allTargets.length, 4)
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
