#!/usr/bin/env node
import fs from 'node:fs'
import { spawn } from 'node:child_process'
import path from 'node:path'

let args = process.argv.slice(2)
const aliasMap = new Map([
  [
    'build:linux',
    ['build', '--target', 'x86_64-unknown-linux-gnu', '--config', 'src-tauri/tauri.linux.conf.json']
  ],
  [
    'build:linux:deb',
    [
      'build',
      '--target',
      'x86_64-unknown-linux-gnu',
      '--config',
      'src-tauri/tauri.linux.conf.json',
      '--',
      '--bundle',
      'deb'
    ]
  ],
  [
    'build:linux:appimage',
    [
      'build',
      '--target',
      'x86_64-unknown-linux-gnu',
      '--config',
      'src-tauri/tauri.linux.conf.json',
      '--',
      '--bundle',
      'appimage'
    ]
  ],
  [
    'build:windows',
    ['build', '--target', 'x86_64-pc-windows-msvc', '--config', 'src-tauri/tauri.windows.conf.json']
  ],
  [
    'build:macos',
    ['build', '--target', 'aarch64-apple-darwin', '--config', 'src-tauri/tauri.macos.arm64.conf.json']
  ],
  [
    'build:macos:intel',
    ['build', '--target', 'x86_64-apple-darwin', '--config', 'src-tauri/tauri.macos.x64.conf.json']
  ],
  [
    'build:macos:dmg',
    [
      'build',
      '--target',
      'aarch64-apple-darwin',
      '--config',
      'src-tauri/tauri.macos.arm64.conf.json',
      '--',
      '--bundle',
      'dmg'
    ]
  ],
  [
    'build:macos:app',
    [
      'build',
      '--target',
      'aarch64-apple-darwin',
      '--config',
      'src-tauri/tauri.macos.arm64.conf.json',
      '--',
      '--bundle',
      'app'
    ]
  ]
])

const initialCommand = args[0] ?? ''
if (aliasMap.has(initialCommand)) {
  const mapped = aliasMap.get(initialCommand) ?? []
  args = [...mapped, ...args.slice(1)]
}

const command = args[0] ?? ''
const hasConfig = hasArg(args, '--config') || hasArg(args, '-c')

if (command === 'build' && !hasConfig) {
  const configPath = resolveConfigPath(args)
  if (configPath) {
    args = insertBeforeSeparator(args, ['--config', configPath])
  }
}
const shouldFetch = command === 'dev' || command === 'build'

if (shouldFetch) {
  const targetInfo = resolveTargetInfo(args)
  const platform = targetInfo?.platform ?? normalizePlatform(process.platform)
  const arch = targetInfo?.arch ?? normalizeArch(process.arch)

  if (platform && arch) {
    const fetchExitCode = await runCommand(process.execPath, [
      path.resolve('scripts', 'fetch-kernel.mjs'),
      '--platform',
      platform,
      '--arch',
      arch,
      '--skip-existing'
    ])
    if (fetchExitCode !== 0) {
      process.exit(fetchExitCode)
    }
  }
}

const tauriCommand = resolveTauriCommand(args)
const exitCode = await runCommand(tauriCommand.command, tauriCommand.args, {
  shell: tauriCommand.shell
})
process.exit(exitCode)

function resolveTargetInfo(argv) {
  const targetIndex = argv.findIndex((item) => item === '--target' || item === '-t')
  if (targetIndex === -1) {
    return null
  }

  const target = argv[targetIndex + 1]
  if (!target) {
    return null
  }

  const parts = target.split('-')
  if (parts.length < 2) {
    return null
  }

  const archToken = parts[0]
  const osToken = parts.find((part) => ['windows', 'darwin', 'linux'].includes(part))

  return {
    platform:
      osToken === 'darwin'
        ? 'macos'
        : osToken === 'windows'
          ? 'windows'
          : osToken === 'linux'
            ? 'linux'
            : null,
    arch: normalizeArch(archToken)
  }
}

function resolveConfigPath(argv) {
  const targetInfo = resolveTargetInfo(argv)
  const platform = targetInfo?.platform ?? normalizePlatform(process.platform)
  const arch = targetInfo?.arch ?? normalizeArch(process.arch)

  if (platform === 'windows') {
    return 'src-tauri/tauri.windows.conf.json'
  }
  if (platform === 'linux') {
    return 'src-tauri/tauri.linux.conf.json'
  }
  if (platform === 'macos') {
    if (arch === 'arm64') {
      return 'src-tauri/tauri.macos.arm64.conf.json'
    }
    return 'src-tauri/tauri.macos.x64.conf.json'
  }
  return null
}

function normalizePlatform(raw) {
  if (raw === 'win32' || raw === 'windows') return 'windows'
  if (raw === 'linux') return 'linux'
  if (raw === 'darwin' || raw === 'macos') return 'macos'
  return null
}

function normalizeArch(raw) {
  if (raw === 'x64' || raw === 'amd64' || raw === 'x86_64') return 'amd64'
  if (raw === 'ia32' || raw === 'x86' || raw === 'i686' || raw === '386') return '386'
  if (raw === 'arm64' || raw === 'aarch64') return 'arm64'
  if (raw === 'arm' || raw === 'armv5') return 'armv5'
  return null
}

function hasArg(argv, flag) {
  return argv.includes(flag)
}

function insertBeforeSeparator(argv, toInsert) {
  const separatorIndex = argv.indexOf('--')
  if (separatorIndex === -1) {
    return [...argv, ...toInsert]
  }
  return [
    ...argv.slice(0, separatorIndex),
    ...toInsert,
    ...argv.slice(separatorIndex)
  ]
}

function resolveTauriCommand(argv) {
  const binDir = path.resolve('node_modules', '.bin')
  if (process.platform === 'win32') {
    const cmdPath = path.join(binDir, 'tauri.cmd')
    if (fs.existsSync(cmdPath)) {
      return { command: cmdPath, args: argv, shell: true }
    }
    const exePath = path.join(binDir, 'tauri.exe')
    if (fs.existsSync(exePath)) {
      return { command: exePath, args: argv, shell: false }
    }
  } else {
    const binPath = path.join(binDir, 'tauri')
    if (fs.existsSync(binPath)) {
      return { command: binPath, args: argv, shell: false }
    }
  }

  return {
    command: 'pnpm',
    args: ['exec', 'tauri', ...argv],
    shell: process.platform === 'win32'
  }
}

function runCommand(command, commandArgs, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, commandArgs, {
      stdio: 'inherit',
      shell: options.shell ?? false
    })
    child.on('error', reject)
    child.on('close', (code) => resolve(code ?? 1))
  })
}
