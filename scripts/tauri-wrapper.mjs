#!/usr/bin/env node
import fs from 'node:fs'
import fsPromises from 'node:fs/promises'
import { spawn } from 'node:child_process'
import path from 'node:path'
import { pathToFileURL } from 'node:url'
import {
  getKernelResourceMap,
  resolveKernelTarget,
  resolveKernelTargetForHost,
  resolveKernelTargetFromRustTarget
} from './kernel-targets.mjs'

const WINDOWS_TARGET = requireTarget('windows', 'amd64')
const LINUX_TARGET = requireTarget('linux', 'amd64')
const MACOS_ARM64_TARGET = requireTarget('macos', 'arm64')
const MACOS_AMD64_TARGET = requireTarget('macos', 'amd64')

const aliasMap = new Map([
  ['build:linux', ['build', '--target', LINUX_TARGET.tauriTarget]],
  [
    'build:linux:deb',
    ['build', '--target', LINUX_TARGET.tauriTarget, '--bundles', 'deb']
  ],
  [
    'build:linux:appimage',
    ['build', '--target', LINUX_TARGET.tauriTarget, '--bundles', 'appimage']
  ],
  [
    'build:linux:rpm',
    ['build', '--target', LINUX_TARGET.tauriTarget, '--bundles', 'rpm']
  ],
  ['build:windows', ['build', '--target', WINDOWS_TARGET.tauriTarget]],
  ['build:macos', ['build', '--target', MACOS_ARM64_TARGET.tauriTarget]],
  ['build:macos:intel', ['build', '--target', MACOS_AMD64_TARGET.tauriTarget]],
  [
    'build:macos:dmg',
    ['build', '--target', MACOS_ARM64_TARGET.tauriTarget, '--bundles', 'dmg']
  ],
  [
    'build:macos:app',
    ['build', '--target', MACOS_ARM64_TARGET.tauriTarget, '--bundles', 'app']
  ]
])

export function applyAliasArgs(rawArgs) {
  const initialCommand = rawArgs[0] ?? ''
  if (!aliasMap.has(initialCommand)) {
    return [...rawArgs]
  }
  const mapped = aliasMap.get(initialCommand) ?? []
  return [...mapped, ...rawArgs.slice(1)]
}

export async function main(rawArgs = process.argv.slice(2)) {
  let args = applyAliasArgs(rawArgs)

  const command = args[0] ?? ''
  const target = resolveTargetFromArgs(args)
  const hasConfig = hasOption(args, '--config', '-c')
  const shouldFetch = command === 'dev' || command === 'build'

  if (shouldFetch && !target) {
    console.error(
      'Unsupported kernel target. Please set a supported --target or extend scripts/kernel-targets.mjs.'
    )
    return 1
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
      return fetchExitCode
    }
  }

  const tauriCommand = resolveTauriCommand(args)
  return runCommand(tauriCommand.command, tauriCommand.args, {
    shell: tauriCommand.shell
  })
}

export function requireTarget(platform, arch) {
  const target = resolveKernelTarget(platform, arch)
  if (!target) {
    throw new Error(`Kernel target not configured: ${platform}/${arch}`)
  }
  return target
}

export function resolveTargetFromArgs(argv) {
  const rustTarget = getOptionValue(argv, '--target', '-t')
  if (rustTarget) {
    return resolveKernelTargetFromRustTarget(rustTarget)
  }
  return resolveKernelTargetForHost()
}

export async function generateBuildConfig(target) {
  const generatedDir = path.resolve('src-tauri', '.generated')
  await fsPromises.mkdir(generatedDir, { recursive: true })

  const generatedConfigPath = path.join(
    generatedDir,
    `tauri.kernel.${target.platform}.${target.arch}.conf.json`
  )
  const config = {
    bundle: {
      // 仅注入当前 target 的内核资源，避免多平台资源被一并打包。
      resources: getKernelResourceMap(
        target.platform,
        target.arch,
        path.resolve('src-tauri', 'resources', 'kernel')
      )
    }
  }
  await fsPromises.writeFile(generatedConfigPath, `${JSON.stringify(config, null, 2)}\n`, 'utf8')
  return toPosixRelativePath(generatedConfigPath)
}

export function toPosixRelativePath(filePath) {
  return path.relative(process.cwd(), filePath).split(path.sep).join('/')
}

export function hasOption(argv, longFlag, shortFlag) {
  return argv.some(
    (item) =>
      item === longFlag ||
      item === shortFlag ||
      item.startsWith(`${longFlag}=`) ||
      item.startsWith(`${shortFlag}=`)
  )
}

export function getOptionValue(argv, longFlag, shortFlag) {
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

export function insertBeforeSeparator(argv, toInsert) {
  const separatorIndex = argv.indexOf('--')
  if (separatorIndex === -1) {
    return [...argv, ...toInsert]
  }
  return [...argv.slice(0, separatorIndex), ...toInsert, ...argv.slice(separatorIndex)]
}

export function resolveTauriCommand(argv) {
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

export function runCommand(command, commandArgs, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, commandArgs, {
      stdio: 'inherit',
      shell: options.shell ?? false
    })
    child.on('error', reject)
    child.on('close', (code) => resolve(code ?? 1))
  })
}

export function wrapWindowsCmd(command, args) {
  const commandLine = [quoteWindowsArg(command), ...args.map(quoteWindowsArg)].join(' ')
  return {
    command: 'cmd.exe',
    args: ['/d', '/s', '/c', commandLine],
    shell: false
  }
}

export function quoteWindowsArg(arg) {
  if (arg.length === 0) {
    return '""'
  }
  if (!/[ \t"]/u.test(arg)) {
    return arg
  }
  const escaped = arg.replace(/(\\*)"/g, '$1$1\\"').replace(/(\\+)$/g, '$1$1')
  return `"${escaped}"`
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const exitCode = await main()
  process.exit(exitCode)
}
