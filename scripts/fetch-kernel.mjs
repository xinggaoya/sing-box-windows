#!/usr/bin/env node
import fs from 'node:fs'
import fsPromises from 'node:fs/promises'
import path from 'node:path'
import os from 'node:os'
import { spawn } from 'node:child_process'
import { pipeline } from 'node:stream/promises'
import { Readable } from 'node:stream'
import { pathToFileURL } from 'node:url'
import {
  getAllKernelTargets,
  normalizeArch,
  normalizePlatform,
  resolveKernelTarget
} from './kernel-targets.mjs'

export async function main(rawArgs = process.argv.slice(2)) {
  const args = parseArgs(rawArgs)
  if (args.help) {
    printHelp()
    return 0
  }

  const baseDir = args.out
    ? path.resolve(args.out)
    : path.resolve('src-tauri', 'resources', 'kernel')
  let resolvedVersion = args.version ?? null
  const skipExisting = Boolean(args['skip-existing'] || args.skipExisting)
  const force = Boolean(args.force)
  const targets = resolveRequestedTargets(args)

  if (targets.length === 0) {
    console.error('Unsupported platform/arch. Use --platform and --arch.')
    return 1
  }

  const errors = []
  for (const target of targets) {
    try {
      await fetchKernel(target, resolvedVersion, baseDir, {
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
    return 1
  }

  return 0
}

export function parseArgs(rawArgs) {
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

export function resolveRequestedTargets(
  args,
  host = { platform: process.platform, arch: process.arch }
) {
  const isAll = args.all || args.platform === 'all'
  if (isAll) {
    return getAllKernelTargets()
  }
  return [
    resolveKernelTarget(
      normalizePlatform(args.platform ?? host.platform),
      normalizeArch(args.arch ?? host.arch)
    )
  ].filter(Boolean)
}

export function printHelp() {
  console.log(`Usage:
  node scripts/fetch-kernel.mjs [--all] [--platform windows|linux|macos] [--arch amd64|arm64|386|armv5] [--version x.y.z] [--track stable|oldstable|beta|testing] [--out path] [--skip-existing] [--force]

Examples:
  node scripts/fetch-kernel.mjs --platform windows --arch amd64
  node scripts/fetch-kernel.mjs --all
  node scripts/fetch-kernel.mjs --platform macos --arch arm64 --version 1.12.10
  node scripts/fetch-kernel.mjs --all --skip-existing
  node scripts/fetch-kernel.mjs --all --track oldstable   # 上一稳定版（适合兼容性回退）
  node scripts/fetch-kernel.mjs --all --track beta        # 预发版
`)
}

export function buildFilename(platformName, archName, versionName) {
  if (platformName === 'windows') {
    return `sing-box-${versionName}-windows-${archName}.zip`
  }
  if (platformName === 'macos') {
    return `sing-box-${versionName}-darwin-${archName}.tar.gz`
  }
  return `sing-box-${versionName}-linux-${archName}.tar.gz`
}

export function buildDownloadUrls(versionName, filenameName) {
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

export async function fetchKernel(target, version, kernelBaseDir, options = {}) {
  const targetDir = path.join(kernelBaseDir, target.platform, target.arch)
  const targetExecutable = path.join(targetDir, target.executable)
  const versionPath = path.join(targetDir, 'version.txt')

  await fsPromises.mkdir(targetDir, { recursive: true })

  if (
    options.skipExisting &&
    !options.force &&
    fs.existsSync(targetExecutable) &&
    fs.existsSync(versionPath)
  ) {
    console.log(`[${target.platform}/${target.arch}] Exists, skipping download.`)
    return
  }

  const resolvedTargetVersion = version ?? (await options.getVersion?.())
  if (!resolvedTargetVersion) {
    throw new Error(`[${target.platform}/${target.arch}] Missing version info.`)
  }

  const filename = buildFilename(target.platform, target.arch, resolvedTargetVersion)
  const downloadUrls = buildDownloadUrls(resolvedTargetVersion, filename)
  const tempDir = await fsPromises.mkdtemp(path.join(os.tmpdir(), 'sing-box-'))
  const archivePath = path.join(tempDir, filename)
  const extractDir = path.join(tempDir, 'extract')
  await fsPromises.mkdir(extractDir, { recursive: true })

  let downloaded = false
  for (const url of downloadUrls) {
    try {
      console.log(`[${target.platform}/${target.arch}] Downloading: ${url}`)
      await downloadFile(url, archivePath)
      downloaded = true
      break
    } catch (error) {
      console.warn(
        `[${target.platform}/${target.arch}] Download failed: ${error?.message ?? error}`
      )
    }
  }

  if (!downloaded) {
    await cleanupTemp(tempDir)
    throw new Error(`[${target.platform}/${target.arch}] All download sources failed.`)
  }

  await extractArchive(archivePath, extractDir)
  const foundExecutable = await findFile(extractDir, path.basename(targetExecutable))

  if (!foundExecutable) {
    await cleanupTemp(tempDir)
    throw new Error(`[${target.platform}/${target.arch}] Executable not found in archive.`)
  }

  await fsPromises.copyFile(foundExecutable, targetExecutable)
  if (target.platform !== 'windows') {
    await fsPromises.chmod(targetExecutable, 0o755)
  }

  await fsPromises.writeFile(versionPath, `${resolvedTargetVersion}\n`, 'utf8')

  await cleanupTemp(tempDir)
  console.log(`[${target.platform}/${target.arch}] Saved: ${targetExecutable}`)
}

export async function getLatestVersion(options = {}) {
  // sing-box 1.14 起 Linux/Docker 有 4 个发布轨：
  // - stable：当前稳定版（默认），走 `releases/latest`
  // - oldstable：上一稳定版（兼容性回退）
  // - beta：稳定预发版
  // - testing：testing 分支
  const track = (options.track ?? 'stable').toLowerCase();
  const includePrerelease = track === 'beta' || track === 'testing';

  // oldstable/beta/testing：拉取 releases 列表后本地筛选
  // stable：直接走 `releases/latest`
  if (track === 'stable') {
    return await getLatestFromEndpoint('https://api.github.com/repos/SagerNet/sing-box/releases/latest');
  }

  return await getLatestFromReleases(includePrerelease, track);
}

async function getLatestFromEndpoint(url) {
  const apiSources = [
    { url, withAuth: true },
    {
      url: `https://v6.gh-proxy.com/${url}`,
      withAuth: false
    },
    {
      url: `https://gh-proxy.com/${url}`,
      withAuth: false
    },
    {
      url: `https://ghfast.top/${url}`,
      withAuth: false
    }
  ];

  for (const source of apiSources) {
    try {
      const res = await fetch(source.url, {
        headers: buildGithubHeaders(source.withAuth),
        redirect: 'follow'
      });
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`);
      }
      const data = await res.json();
      const version = normalizeVersionTag(data?.tag_name);
      if (version) {
        console.log(`[track] resolved version ${version} from ${source.url}`);
        return version;
      }
    } catch (error) {
      console.warn(`Latest version fetch failed: ${error?.message ?? error}`);
    }
  }

  // Fallback to release page scraping
  return await scrapeLatestFromReleasePage();
}

async function getLatestFromReleases(includePrerelease, track) {
  const apiSources = [
    {
      url: 'https://api.github.com/repos/SagerNet/sing-box/releases?per_page=100',
      withAuth: true
    },
    {
      url: 'https://v6.gh-proxy.com/https://api.github.com/repos/SagerNet/sing-box/releases?per_page=100',
      withAuth: false
    },
    {
      url: 'https://gh-proxy.com/https://api.github.com/repos/SagerNet/sing-box/releases?per_page=100',
      withAuth: false
    },
    {
      url: 'https://ghfast.top/https://api.github.com/repos/SagerNet/sing-box/releases?per_page=100',
      withAuth: false
    }
  ];

  for (const source of apiSources) {
    try {
      const res = await fetch(source.url, {
        headers: buildGithubHeaders(source.withAuth),
        redirect: 'follow'
      });
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`);
      }
      const releases = await res.json();
      if (!Array.isArray(releases) || releases.length === 0) {
        throw new Error('empty releases list');
      }
      const version = selectVersionFromReleases(releases, track, includePrerelease);
      if (version) {
        console.log(`[track=${track}] resolved version ${version} from ${source.url}`);
        return version;
      }
    } catch (error) {
      console.warn(`Releases list fetch failed: ${error?.message ?? error}`);
    }
  }

  throw new Error(
    `Unable to fetch releases list for track=${track}. If running in CI, set SING_BOX_GITHUB_TOKEN for higher API rate limits.`
  );
}

/// 从已排序（newest-first）的 releases 数组里按 track 选一个版本。
/// - oldstable：找版本号第二大的 stable（major.minor 不同的最大 stable）
/// - beta：第一个 prerelease
/// - testing：第一个 prerelease（beta/testing 共用 same channel）
/// - stable：第一个 stable release
function selectVersionFromReleases(releases, track, includePrerelease) {
  const sorted = [...releases].sort((a, b) => {
    const at = a?.published_at ?? a?.created_at ?? '';
    const bt = b?.published_at ?? b?.created_at ?? '';
    return bt.localeCompare(at); // descending
  });

  if (track === 'stable') {
    const found = sorted.find((r) => !r.prerelease);
    return found ? normalizeVersionTag(found.tag_name) : null;
  }

  if (track === 'oldstable') {
    // 1) 找最新的 major.minor 组（>= 1.14.x）
    // 2) 再找它前一个 major.minor 的最新 stable
    const stableReleases = sorted.filter((r) => !r.prerelease);
    if (stableReleases.length < 2) {
      // 兜底：只有一个 stable 时直接用
      return stableReleases[0] ? normalizeVersionTag(stableReleases[0].tag_name) : null;
    }
    const newest = normalizeVersionTag(stableReleases[0].tag_name);
    const newestMajorMinor = newest?.split('.').slice(0, 2).join('.');
    const olderMajorMinorReleases = stableReleases
      .map((r) => ({ release: r, version: normalizeVersionTag(r.tag_name) }))
      .filter(({ version }) => {
        const mm = version?.split('.').slice(0, 2).join('.');
        return mm && mm !== newestMajorMinor;
      });
    if (olderMajorMinorReleases.length === 0) {
      return newest;
    }
    return olderMajorMinorReleases[0].version;
  }

  if (track === 'beta' || track === 'testing') {
    if (!includePrerelease) {
      return null;
    }
    const found = sorted.find((r) => r.prerelease);
    return found ? normalizeVersionTag(found.tag_name) : null;
  }

  return null;
}

async function scrapeLatestFromReleasePage() {
  const releasePageSources = [
    'https://github.com/SagerNet/sing-box/releases/latest',
    'https://v6.gh-proxy.com/https://github.com/SagerNet/sing-box/releases/latest',
    'https://gh-proxy.com/https://github.com/SagerNet/sing-box/releases/latest',
    'https://ghfast.top/https://github.com/SagerNet/sing-box/releases/latest'
  ];

  for (const url of releasePageSources) {
    try {
      const res = await fetch(url, {
        headers: buildGithubHeaders(false),
        redirect: 'follow'
      });
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`);
      }
      const fromFinalUrl = parseVersionFromReleaseUrl(res.url);
      if (fromFinalUrl) {
        return fromFinalUrl;
      }
      const html = await res.text();
      const fromHtml = parseVersionFromReleaseHtml(html);
      if (fromHtml) {
        return fromHtml;
      }
    } catch (error) {
      console.warn(`Latest release page fetch failed: ${error?.message ?? error}`);
    }
  }

  throw new Error('Unable to fetch latest version (fallback also failed).');
}

export function normalizeVersionTag(tag) {
  if (!tag || typeof tag !== 'string') {
    return null
  }
  const trimmed = tag.trim()
  if (!trimmed) {
    return null
  }
  return trimmed.startsWith('v') ? trimmed.slice(1) : trimmed
}

export function parseVersionFromReleaseUrl(url) {
  if (!url || typeof url !== 'string') {
    return null
  }
  const match = url.match(/\/releases\/tag\/v?([^/?#]+)$/i)
  if (!match || !match[1]) {
    return null
  }
  return normalizeVersionTag(match[1])
}

export function parseVersionFromReleaseHtml(html) {
  if (!html || typeof html !== 'string') {
    return null
  }
  const match = html.match(/\/SagerNet\/sing-box\/releases\/tag\/v?([A-Za-z0-9._-]+)/i)
  if (!match || !match[1]) {
    return null
  }
  return normalizeVersionTag(match[1])
}

export function buildGithubHeaders(withAuth) {
  const headers = {
    Accept: 'application/vnd.github+json',
    'User-Agent': 'sing-box-windows'
  }
  if (withAuth) {
    const token =
      process.env.SING_BOX_GITHUB_TOKEN || process.env.GITHUB_TOKEN || process.env.GH_TOKEN
    if (token) {
      headers.Authorization = `Bearer ${token}`
      headers['X-GitHub-Api-Version'] = '2022-11-28'
    }
  }
  return headers
}

export async function downloadFile(url, destination) {
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

export async function extractArchive(archivePath, outputDir) {
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

export async function runCommand(command, commandArgs) {
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

export async function findFile(rootDir, fileName) {
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

export async function cleanupTemp(dir) {
  try {
    await fsPromises.rm(dir, { recursive: true, force: true })
  } catch {
    // best-effort cleanup
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const exitCode = await main()
  process.exit(exitCode)
}
