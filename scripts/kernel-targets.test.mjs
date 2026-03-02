import test from 'node:test'
import assert from 'node:assert/strict'
import path from 'node:path'
import { getKernelResourceMap } from './kernel-targets.mjs'

test('getKernelResourceMap 仅返回当前目标的内核资源映射', () => {
  const root = path.join('tmp', 'kernel-root')
  const map = getKernelResourceMap('windows', 'amd64', root)

  const keys = Object.keys(map)
  assert.equal(keys.length, 2)

  const executable = path.resolve(root, 'windows', 'amd64', 'sing-box.exe')
  const versionFile = path.resolve(root, 'windows', 'amd64', 'version.txt')

  assert.equal(map[executable], 'kernel/windows/amd64/sing-box.exe')
  assert.equal(map[versionFile], 'kernel/windows/amd64/version.txt')
})

test('getKernelResourceMap 对不支持的目标返回空对象', () => {
  const map = getKernelResourceMap('linux', 'arm64', 'tmp/kernel-root')
  assert.deepEqual(map, {})
})
