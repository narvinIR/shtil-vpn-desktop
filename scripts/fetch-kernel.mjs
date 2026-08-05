#!/usr/bin/env node
import fs from 'node:fs'
import fsPromises from 'node:fs/promises'
import path from 'node:path'
import os from 'node:os'
import crypto from 'node:crypto'
import { spawn } from 'node:child_process'
import { pipeline } from 'node:stream/promises'
import { Readable } from 'node:stream'
import { pathToFileURL } from 'node:url'
import {
  getAllKernelTargets,
  normalizeArch,
  normalizePlatform,
  resolveKernelTarget,
  KERNEL_VERSION
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
  if (args.version && args.version !== KERNEL_VERSION) {
    console.error(
      `Kernel version is pinned to ${KERNEL_VERSION} together with its checksums. ` +
        'Update scripts/kernel-targets.mjs instead of passing --version.'
    )
    return 1
  }

  const resolvedVersion = KERNEL_VERSION
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
      await fetchKernel(target, resolvedVersion, baseDir, { skipExisting, force })
    } catch (error) {
      console.error(error?.message ?? error)
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
  node scripts/fetch-kernel.mjs [--all] [--platform windows|linux|macos] [--arch amd64|arm64|386|armv5] [--out path] [--skip-existing] [--force]

Ядро берётся версии ${KERNEL_VERSION} — она закреплена вместе с контрольными
суммами в scripts/kernel-targets.mjs. Другая версия = другие суммы, поэтому
--version здесь нет: правится реестр целей.

Examples:
  node scripts/fetch-kernel.mjs --platform windows --arch amd64
  node scripts/fetch-kernel.mjs --all
  node scripts/fetch-kernel.mjs --all --skip-existing
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

// Один источник — тот, кто ядро выпускает. Чужие зеркала (в том числе китайские,
// стоявшие тут первыми) отдают файл, который потом работает у клиента от имени
// администратора: подменить его = подменить VPN у всех.
export function buildDownloadUrl(versionName, filenameName) {
  return `https://github.com/SagerNet/sing-box/releases/download/v${versionName}/${filenameName}`
}

export async function sha256OfFile(filePath) {
  const hash = crypto.createHash('sha256')
  await pipeline(fs.createReadStream(filePath), hash)
  return hash.digest('hex')
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

  const resolvedTargetVersion = version
  if (!resolvedTargetVersion) {
    throw new Error(`[${target.platform}/${target.arch}] Missing version info.`)
  }

  const filename = buildFilename(target.platform, target.arch, resolvedTargetVersion)
  const downloadUrl = buildDownloadUrl(resolvedTargetVersion, filename)
  const tempDir = await fsPromises.mkdtemp(path.join(os.tmpdir(), 'sing-box-'))
  const archivePath = path.join(tempDir, filename)
  const extractDir = path.join(tempDir, 'extract')
  await fsPromises.mkdir(extractDir, { recursive: true })

  try {
    console.log(`[${target.platform}/${target.arch}] Downloading: ${downloadUrl}`)
    await downloadFile(downloadUrl, archivePath)
  } catch (error) {
    await cleanupTemp(tempDir)
    throw new Error(
      `[${target.platform}/${target.arch}] Download failed: ${error?.message ?? error}`
    )
  }

  const actualChecksum = await sha256OfFile(archivePath)
  if (actualChecksum !== target.sha256) {
    await cleanupTemp(tempDir)
    throw new Error(
      `[${target.platform}/${target.arch}] Checksum mismatch for ${filename}: ` +
        `expected ${target.sha256}, got ${actualChecksum}.`
    )
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

export async function downloadFile(url, destination) {
  const res = await fetch(url, {
    headers: { 'User-Agent': 'shtil-vpn-desktop' },
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
