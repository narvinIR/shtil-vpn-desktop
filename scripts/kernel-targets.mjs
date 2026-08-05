import path from 'node:path'

// Ядро, которое уезжает клиентам внутри установщика. Версия закреплена, а не
// «последняя на момент сборки»: у закреплённой есть контрольная сумма, и подмена
// файла роняет сборку вместо того, чтобы доехать до людей. Обновление ядра —
// осознанный шаг: поднять версию и переписать суммы со страницы выпуска
// github.com/SagerNet/sing-box/releases.
const KERNEL_VERSION = '1.13.16'

const KERNEL_TARGETS = Object.freeze([
  {
    platform: 'windows',
    arch: 'amd64',
    executable: 'sing-box.exe',
    tauriTarget: 'x86_64-pc-windows-msvc',
    sha256: '6cbf90ec4ee87122ffce09b73928fb31e763bc1c75a119f79c61d24734c78807'
  },
  {
    platform: 'windows',
    arch: 'arm64',
    executable: 'sing-box.exe',
    tauriTarget: 'aarch64-pc-windows-msvc',
    sha256: '8412e9751a776a1cd5138fde8a6b60784af91b0fe596cba1b6efcd05144ef511'
  },
  {
    platform: 'linux',
    arch: 'amd64',
    executable: 'sing-box',
    tauriTarget: 'x86_64-unknown-linux-gnu',
    sha256: 'e37c312859dfa84cba148f41072ff6369f08361ae91d622dc1fd3aab49611a8d'
  },
  {
    platform: 'macos',
    arch: 'arm64',
    executable: 'sing-box',
    tauriTarget: 'aarch64-apple-darwin',
    sha256: '32fa21fd75ad62d86a2dcb7e0be77359c35e12798cdbb6a0e30654ef487d90d6'
  },
  {
    platform: 'macos',
    arch: 'amd64',
    executable: 'sing-box',
    tauriTarget: 'x86_64-apple-darwin',
    sha256: '2bfad58d034e280c773e194be03649555e5a7040c48b559dd0898ad293fe793d'
  }
])

export { KERNEL_TARGETS, KERNEL_VERSION }

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

export function getKernelResourceMap(platformRaw, archRaw, kernelBaseDir) {
  const target = resolveKernelTarget(platformRaw, archRaw)
  if (!target) {
    return {}
  }

  const baseDir = path.resolve(
    kernelBaseDir ?? path.resolve('src-tauri', 'resources', 'kernel')
  )
  const sourceBase = path.join(baseDir, target.platform, target.arch)
  const destinationBase = `kernel/${target.platform}/${target.arch}`

  return {
    [path.join(sourceBase, target.executable)]: `${destinationBase}/${target.executable}`,
    [path.join(sourceBase, 'version.txt')]: `${destinationBase}/version.txt`
  }
}

export function getAllKernelTargets() {
  return [...KERNEL_TARGETS]
}
