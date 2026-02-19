#!/usr/bin/env node
import fs from 'node:fs'
import fsPromises from 'node:fs/promises'
import { spawn } from 'node:child_process'
import path from 'node:path'
import {
  getKernelResourcePaths,
  resolveKernelTarget,
  resolveKernelTargetForHost,
  resolveKernelTargetFromRustTarget
} from './kernel-targets.mjs'

const WINDOWS_TARGET = requireTarget('windows', 'amd64')
const LINUX_TARGET = requireTarget('linux', 'amd64')
const MACOS_ARM64_TARGET = requireTarget('macos', 'arm64')
const MACOS_AMD64_TARGET = requireTarget('macos', 'amd64')

let args = process.argv.slice(2)
const aliasMap = new Map([
  ['build:linux', ['build', '--target', LINUX_TARGET.tauriTarget]],
  [
    'build:linux:deb',
    ['build', '--target', LINUX_TARGET.tauriTarget, '--', '--bundle', 'deb']
  ],
  [
    'build:linux:appimage',
    ['build', '--target', LINUX_TARGET.tauriTarget, '--', '--bundle', 'appimage']
  ],
  ['build:windows', ['build', '--target', WINDOWS_TARGET.tauriTarget]],
  ['build:macos', ['build', '--target', MACOS_ARM64_TARGET.tauriTarget]],
  ['build:macos:intel', ['build', '--target', MACOS_AMD64_TARGET.tauriTarget]],
  [
    'build:macos:dmg',
    ['build', '--target', MACOS_ARM64_TARGET.tauriTarget, '--', '--bundle', 'dmg']
  ],
  [
    'build:macos:app',
    ['build', '--target', MACOS_ARM64_TARGET.tauriTarget, '--', '--bundle', 'app']
  ]
])

const initialCommand = args[0] ?? ''
if (aliasMap.has(initialCommand)) {
  const mapped = aliasMap.get(initialCommand) ?? []
  args = [...mapped, ...args.slice(1)]
}

const command = args[0] ?? ''
const target = resolveTargetFromArgs(args)
const hasConfig = hasOption(args, '--config', '-c')
const shouldFetch = command === 'dev' || command === 'build'

if (shouldFetch && !target) {
  console.error(
    'Unsupported kernel target. Please set a supported --target or extend scripts/kernel-targets.mjs.'
  )
  process.exit(1)
}

if (command === 'build' && !hasConfig && target) {
  const generatedConfigPath = await generateBuildConfig(target)
  args = insertBeforeSeparator(args, ['--config', generatedConfigPath])
}

if (shouldFetch && target) {
  const fetchMode = String(process.env.SING_BOX_KERNEL_FETCH_MODE ?? 'skip').toLowerCase()
  const fetchArgs = [
    path.resolve('scripts', 'fetch-kernel.mjs'),
    '--platform',
    target.platform,
    '--arch',
    target.arch
  ]
  if (fetchMode === 'force') {
    fetchArgs.push('--force')
  } else {
    fetchArgs.push('--skip-existing')
  }

  const fetchExitCode = await runCommand(process.execPath, fetchArgs)
  if (fetchExitCode !== 0) {
    process.exit(fetchExitCode)
  }
}

const tauriCommand = resolveTauriCommand(args)
const exitCode = await runCommand(tauriCommand.command, tauriCommand.args, {
  shell: tauriCommand.shell
})
process.exit(exitCode)

function requireTarget(platform, arch) {
  const target = resolveKernelTarget(platform, arch)
  if (!target) {
    throw new Error(`Kernel target not configured: ${platform}/${arch}`)
  }
  return target
}

function resolveTargetFromArgs(argv) {
  const rustTarget = getOptionValue(argv, '--target', '-t')
  if (rustTarget) {
    return resolveKernelTargetFromRustTarget(rustTarget)
  }
  return resolveKernelTargetForHost()
}

async function generateBuildConfig(target) {
  const generatedDir = path.resolve('src-tauri', '.generated')
  await fsPromises.mkdir(generatedDir, { recursive: true })

  const generatedConfigPath = path.join(
    generatedDir,
    `tauri.kernel.${target.platform}.${target.arch}.conf.json`
  )
  const config = {
    bundle: {
      resources: getKernelResourcePaths(target.platform, target.arch)
    }
  }
  await fsPromises.writeFile(generatedConfigPath, `${JSON.stringify(config, null, 2)}\n`, 'utf8')
  return toPosixRelativePath(generatedConfigPath)
}

function toPosixRelativePath(filePath) {
  return path.relative(process.cwd(), filePath).split(path.sep).join('/')
}

function hasOption(argv, longFlag, shortFlag) {
  return argv.some(
    (item) =>
      item === longFlag ||
      item === shortFlag ||
      item.startsWith(`${longFlag}=`) ||
      item.startsWith(`${shortFlag}=`)
  )
}

function getOptionValue(argv, longFlag, shortFlag) {
  for (let i = 0; i < argv.length; i += 1) {
    const item = argv[i]
    if (item === longFlag || item === shortFlag) {
      return argv[i + 1] ?? null
    }
    if (item.startsWith(`${longFlag}=`)) {
      return item.slice(longFlag.length + 1)
    }
    if (item.startsWith(`${shortFlag}=`)) {
      return item.slice(shortFlag.length + 1)
    }
  }
  return null
}

function insertBeforeSeparator(argv, toInsert) {
  const separatorIndex = argv.indexOf('--')
  if (separatorIndex === -1) {
    return [...argv, ...toInsert]
  }
  return [...argv.slice(0, separatorIndex), ...toInsert, ...argv.slice(separatorIndex)]
}

function resolveTauriCommand(argv) {
  const binDir = path.resolve('node_modules', '.bin')
  if (process.platform === 'win32') {
    const cmdPath = path.join(binDir, 'tauri.cmd')
    if (fs.existsSync(cmdPath)) {
      return wrapWindowsCmd(cmdPath, argv)
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

  if (process.platform === 'win32') {
    return wrapWindowsCmd('pnpm', ['exec', 'tauri', ...argv])
  }

  return {
    command: 'pnpm',
    args: ['exec', 'tauri', ...argv],
    shell: false
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

function wrapWindowsCmd(command, args) {
  const commandLine = [quoteWindowsArg(command), ...args.map(quoteWindowsArg)].join(' ')
  return {
    command: 'cmd.exe',
    args: ['/d', '/s', '/c', commandLine],
    shell: false
  }
}

function quoteWindowsArg(arg) {
  if (arg.length === 0) {
    return '""'
  }
  if (!/[ \t"]/u.test(arg)) {
    return arg
  }
  const escaped = arg.replace(/(\\*)"/g, '$1$1\\"').replace(/(\\+)$/g, '$1$1')
  return `"${escaped}"`
}
