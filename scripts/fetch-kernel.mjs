#!/usr/bin/env node
import fs from 'node:fs'
import fsPromises from 'node:fs/promises'
import path from 'node:path'
import os from 'node:os'
import { spawn } from 'node:child_process'
import { pipeline } from 'node:stream/promises'
import { Readable } from 'node:stream'

const args = parseArgs(process.argv.slice(2))
if (args.help) {
  printHelp()
  process.exit(0)
}

const baseDir = args.out
  ? path.resolve(args.out)
  : path.resolve('src-tauri', 'resources', 'kernel')
let resolvedVersion = args.version ?? null
const isAll = args.all || args.platform === 'all'
const skipExisting = Boolean(args['skip-existing'] || args.skipExisting)
const force = Boolean(args.force)

const targets = isAll
  ? [
      { platform: 'windows', arch: 'amd64' },
      { platform: 'linux', arch: 'amd64' },
      { platform: 'macos', arch: 'arm64' },
      { platform: 'macos', arch: 'amd64' }
    ]
  : [
      {
        platform: normalizePlatform(args.platform ?? process.platform),
        arch: normalizeArch(args.arch ?? process.arch)
      }
    ]

if (targets.some((item) => !item.platform || !item.arch)) {
  console.error('Unsupported platform/arch. Use --platform and --arch.')
  process.exit(1)
}

const errors = []
for (const target of targets) {
  try {
    await fetchKernel(target.platform, target.arch, resolvedVersion, baseDir, {
      skipExisting,
      force,
      getVersion: async () => {
        if (!resolvedVersion) {
          resolvedVersion = await getLatestVersion()
        }
        return resolvedVersion
      }
    })
  } catch (error) {
    errors.push(error)
  }
}

if (errors.length > 0) {
  console.error(`Failed: ${errors.length} target(s).`)
  process.exit(1)
}

function parseArgs(rawArgs) {
  const result = {}
  for (let i = 0; i < rawArgs.length; i += 1) {
    const token = rawArgs[i]
    if (token.startsWith('--')) {
      const [key, value] = token.slice(2).split('=')
      if (value !== undefined) {
        result[key] = value
      } else if (rawArgs[i + 1] && !rawArgs[i + 1].startsWith('--')) {
        result[key] = rawArgs[i + 1]
        i += 1
      } else {
        result[key] = true
      }
    }
  }
  return result
}

function printHelp() {
  console.log(`Usage:
  node scripts/fetch-kernel.mjs [--all] [--platform windows|linux|macos] [--arch amd64|arm64|386|armv5] [--version x.y.z] [--out path] [--skip-existing] [--force]

Examples:
  node scripts/fetch-kernel.mjs --platform windows --arch amd64
  node scripts/fetch-kernel.mjs --all
  node scripts/fetch-kernel.mjs --platform macos --arch arm64 --version 1.12.10
  node scripts/fetch-kernel.mjs --all --skip-existing
`)
}

function normalizePlatform(raw) {
  if (!raw) return null
  if (raw === 'win32' || raw === 'windows') return 'windows'
  if (raw === 'linux') return 'linux'
  if (raw === 'darwin' || raw === 'macos') return 'macos'
  return null
}

function normalizeArch(raw) {
  if (!raw) return null
  if (raw === 'x64' || raw === 'amd64') return 'amd64'
  if (raw === 'ia32' || raw === 'x86' || raw === '386') return '386'
  if (raw === 'arm64' || raw === 'aarch64') return 'arm64'
  if (raw === 'arm' || raw === 'armv5') return 'armv5'
  return null
}

function buildFilename(platformName, archName, versionName) {
  if (platformName === 'windows') {
    return `sing-box-${versionName}-windows-${archName}.zip`
  }
  if (platformName === 'macos') {
    return `sing-box-${versionName}-darwin-${archName}.tar.gz`
  }
  return `sing-box-${versionName}-linux-${archName}.tar.gz`
}

function buildDownloadUrls(versionName, filenameName) {
  const base = `https://github.com/SagerNet/sing-box/releases/download/v${versionName}/${filenameName}`
  return [
    `https://v6.gh-proxy.com/${base}`,
    `https://gh-proxy.com/${base}`,
    `https://ghfast.top/${base}`,
    `https://hub.fastgit.xyz/SagerNet/sing-box/releases/download/v${versionName}/${filenameName}`,
    `https://hub.fgit.cf/SagerNet/sing-box/releases/download/v${versionName}/${filenameName}`,
    `https://cdn.jsdelivr.net/gh/SagerNet/sing-box@releases/download/v${versionName}/${filenameName}`,
    base
  ]
}

async function fetchKernel(platform, arch, version, baseDir, options = {}) {
  const targetDir = path.join(baseDir, platform, arch)
  const targetExecutable = path.join(
    targetDir,
    platform === 'windows' ? 'sing-box.exe' : 'sing-box'
  )
  const versionPath = path.join(targetDir, 'version.txt')

  await fsPromises.mkdir(targetDir, { recursive: true })

  if (
    options.skipExisting &&
    !options.force &&
    fs.existsSync(targetExecutable) &&
    fs.existsSync(versionPath)
  ) {
    console.log(`[${platform}/${arch}] Exists, skipping download.`)
    return
  }

  const resolvedVersion = version ?? (await options.getVersion?.())
  if (!resolvedVersion) {
    throw new Error(`[${platform}/${arch}] Missing version info.`)
  }

  const filename = buildFilename(platform, arch, resolvedVersion)
  const downloadUrls = buildDownloadUrls(resolvedVersion, filename)
  const tempDir = await fsPromises.mkdtemp(path.join(os.tmpdir(), 'sing-box-'))
  const archivePath = path.join(tempDir, filename)
  const extractDir = path.join(tempDir, 'extract')
  await fsPromises.mkdir(extractDir, { recursive: true })

  let downloaded = false
  for (const url of downloadUrls) {
    try {
      console.log(`[${platform}/${arch}] Downloading: ${url}`)
      await downloadFile(url, archivePath)
      downloaded = true
      break
    } catch (error) {
      console.warn(`[${platform}/${arch}] Download failed: ${error?.message ?? error}`)
    }
  }

  if (!downloaded) {
    await cleanupTemp(tempDir)
    throw new Error(`[${platform}/${arch}] All download sources failed.`)
  }

  await extractArchive(archivePath, extractDir)
  const foundExecutable = await findFile(extractDir, path.basename(targetExecutable))

  if (!foundExecutable) {
    await cleanupTemp(tempDir)
    throw new Error(`[${platform}/${arch}] Executable not found in archive.`)
  }

  await fsPromises.copyFile(foundExecutable, targetExecutable)
  if (platform !== 'windows') {
    await fsPromises.chmod(targetExecutable, 0o755)
  }

  await fsPromises.writeFile(versionPath, `${resolvedVersion}\n`, 'utf8')

  await cleanupTemp(tempDir)
  console.log(`[${platform}/${arch}] Saved: ${targetExecutable}`)
}

async function getLatestVersion() {
  const urls = [
    'https://api.github.com/repos/SagerNet/sing-box/releases/latest',
    'https://v6.gh-proxy.com/https://api.github.com/repos/SagerNet/sing-box/releases/latest',
    'https://gh-proxy.com/https://api.github.com/repos/SagerNet/sing-box/releases/latest',
    'https://ghfast.top/https://api.github.com/repos/SagerNet/sing-box/releases/latest'
  ]

  for (const url of urls) {
    try {
      const res = await fetch(url, {
        headers: { 'User-Agent': 'sing-box-windows' }
      })
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`)
      }
      const data = await res.json()
      const tag = data?.tag_name
      if (tag) {
        return tag.startsWith('v') ? tag.slice(1) : tag
      }
    } catch (error) {
      console.warn(`Latest version fetch failed: ${error?.message ?? error}`)
    }
  }

  throw new Error('Unable to fetch latest version.')
}

async function downloadFile(url, destination) {
  const res = await fetch(url, {
    headers: { 'User-Agent': 'sing-box-windows' },
    redirect: 'follow'
  })
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`)
  }

  const body = res.body
  if (!body) {
    throw new Error('Empty response body')
  }

  const fileStream = fs.createWriteStream(destination)
  await pipeline(Readable.fromWeb(body), fileStream)
}

async function extractArchive(archivePath, outputDir) {
  if (archivePath.endsWith('.zip')) {
    try {
      await runCommand('tar', ['-xf', archivePath, '-C', outputDir])
      return
    } catch {
      // Fall back for environments without tar/unzip.
    }

    if (process.platform === 'win32') {
      await runCommand('powershell', [
        '-NoProfile',
        '-Command',
        `Expand-Archive -LiteralPath "${archivePath}" -DestinationPath "${outputDir}" -Force`
      ])
      return
    }

    await runCommand('unzip', ['-q', archivePath, '-d', outputDir])
    return
  }

  if (archivePath.endsWith('.tar.gz')) {
    await runCommand('tar', ['-xzf', archivePath, '-C', outputDir])
    return
  }

  throw new Error(`Unsupported archive: ${archivePath}`)
}

async function runCommand(command, commandArgs) {
  await new Promise((resolve, reject) => {
    const child = spawn(command, commandArgs, { stdio: 'inherit' })
    child.on('error', reject)
    child.on('close', (code) => {
      if (code === 0) {
        resolve()
      } else {
        reject(new Error(`${command} exited with code ${code}`))
      }
    })
  })
}

async function findFile(rootDir, fileName) {
  const entries = await fsPromises.readdir(rootDir, { withFileTypes: true })
  for (const entry of entries) {
    const entryPath = path.join(rootDir, entry.name)
    if (entry.isDirectory()) {
      const nested = await findFile(entryPath, fileName)
      if (nested) return nested
    } else if (entry.isFile() && entry.name === fileName) {
      return entryPath
    }
  }
  return null
}

async function cleanupTemp(dir) {
  try {
    await fsPromises.rm(dir, { recursive: true, force: true })
  } catch {
    // best-effort cleanup
  }
}
