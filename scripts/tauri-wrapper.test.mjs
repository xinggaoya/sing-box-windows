import test from 'node:test'
import assert from 'node:assert/strict'
import path from 'node:path'
import {
  applyAliasArgs,
  getOptionValue,
  hasOption,
  insertBeforeSeparator,
  quoteWindowsArg,
  resolveTargetFromArgs,
  toPosixRelativePath,
  wrapWindowsCmd
} from './tauri-wrapper.mjs'

test('applyAliasArgs 应展开预设构建别名', () => {
  const args = applyAliasArgs(['build:windows'])

  assert.equal(args[0], 'build')
  assert.equal(args[1], '--target')
  assert.equal(args[2], 'x86_64-pc-windows-msvc')
})

test('applyAliasArgs 应支持 Linux RPM 构建别名', () => {
  const args = applyAliasArgs(['build:linux:rpm'])

  assert.deepEqual(args, [
    'build',
    '--target',
    'x86_64-unknown-linux-gnu',
    '--bundles',
    'rpm'
  ])
})

test('hasOption 和 getOptionValue 支持普通与等号参数', () => {
  const args = ['build', '--target=x86_64-pc-windows-msvc', '--config', 'tmp/test.json']

  assert.equal(hasOption(args, '--target', '-t'), true)
  assert.equal(hasOption(args, '--config', '-c'), true)
  assert.equal(getOptionValue(args, '--target', '-t'), 'x86_64-pc-windows-msvc')
  assert.equal(getOptionValue(args, '--config', '-c'), 'tmp/test.json')
})

test('insertBeforeSeparator 会把参数插入到分隔符前', () => {
  assert.deepEqual(
    insertBeforeSeparator(['build', '--', '--bundle', 'msi'], ['--config', 'tmp/test.json']),
    ['build', '--config', 'tmp/test.json', '--', '--bundle', 'msi']
  )

  assert.deepEqual(
    insertBeforeSeparator(['dev'], ['--config', 'tmp/test.json']),
    ['dev', '--config', 'tmp/test.json']
  )
})

test('resolveTargetFromArgs 能根据 Rust target 解析平台信息', () => {
  const target = resolveTargetFromArgs(['build', '--target', 'aarch64-apple-darwin'])

  assert.equal(target?.platform, 'macos')
  assert.equal(target?.arch, 'arm64')
  assert.equal(target?.tauriTarget, 'aarch64-apple-darwin')
})

test('quoteWindowsArg 和 wrapWindowsCmd 会保留空格与引号', () => {
  assert.equal(quoteWindowsArg('plain'), 'plain')
  assert.equal(quoteWindowsArg('hello world'), '"hello world"')
  assert.equal(quoteWindowsArg('say"hi'), '"say\\"hi"')

  const wrapped = wrapWindowsCmd('pnpm', ['exec', 'tauri', '--config', 'tmp/test path.json'])
  assert.equal(wrapped.command, 'cmd.exe')
  assert.deepEqual(wrapped.args.slice(0, 3), ['/d', '/s', '/c'])
  assert.match(wrapped.args[3], /"tmp\/test path\.json"|"tmp\\test path\.json"/)
})

test('toPosixRelativePath 会输出基于工作区的 posix 路径', () => {
  const filePath = path.join(process.cwd(), 'src-tauri', '.generated', 'test.json')
  assert.equal(toPosixRelativePath(filePath), 'src-tauri/.generated/test.json')
})
