const KERNEL_TARGETS = Object.freeze([
  {
    platform: 'windows',
    arch: 'amd64',
    executable: 'sing-box.exe',
    tauriTarget: 'x86_64-pc-windows-msvc'
  },
  {
    platform: 'linux',
    arch: 'amd64',
    executable: 'sing-box',
    tauriTarget: 'x86_64-unknown-linux-gnu'
  },
  {
    platform: 'macos',
    arch: 'arm64',
    executable: 'sing-box',
    tauriTarget: 'aarch64-apple-darwin'
  },
  {
    platform: 'macos',
    arch: 'amd64',
    executable: 'sing-box',
    tauriTarget: 'x86_64-apple-darwin'
  }
])

export function normalizePlatform(raw) {
  if (!raw) return null
  if (raw === 'win32' || raw === 'windows') return 'windows'
  if (raw === 'linux') return 'linux'
  if (raw === 'darwin' || raw === 'macos') return 'macos'
  return null
}

export function normalizeArch(raw) {
  if (!raw) return null
  if (raw === 'x64' || raw === 'amd64' || raw === 'x86_64') return 'amd64'
  if (raw === 'ia32' || raw === 'x86' || raw === 'i686' || raw === '386') return '386'
  if (raw === 'arm64' || raw === 'aarch64') return 'arm64'
  if (raw === 'arm' || raw === 'armv5') return 'armv5'
  return null
}

export function resolveKernelTarget(platformRaw, archRaw) {
  const platform = normalizePlatform(platformRaw)
  const arch = normalizeArch(archRaw)
  if (!platform || !arch) return null
  return (
    KERNEL_TARGETS.find((item) => item.platform === platform && item.arch === arch) ?? null
  )
}

export function resolveKernelTargetForHost() {
  return resolveKernelTarget(process.platform, process.arch)
}

export function resolveKernelTargetFromRustTarget(rustTarget) {
  if (!rustTarget) return null

  const parts = rustTarget.split('-')
  if (parts.length < 2) return null

  const arch = normalizeArch(parts[0])
  const osToken = parts.find((part) => ['windows', 'darwin', 'linux'].includes(part))
  const platform = normalizePlatform(
    osToken === 'darwin' ? 'macos' : osToken === 'windows' ? 'windows' : osToken
  )

  if (!platform || !arch) return null
  return resolveKernelTarget(platform, arch)
}

export function getKernelResourcePaths(platformRaw, archRaw) {
  const target = resolveKernelTarget(platformRaw, archRaw)
  if (!target) {
    return []
  }

  const base = `resources/kernel/${target.platform}/${target.arch}`
  return [`${base}/${target.executable}`, `${base}/version.txt`]
}

export function getAllKernelTargets() {
  return [...KERNEL_TARGETS]
}
