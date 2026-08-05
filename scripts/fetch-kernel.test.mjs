import test from 'node:test'
import assert from 'node:assert/strict'
import {
  buildDownloadUrl,
  buildFilename,
  parseArgs,
  resolveRequestedTargets
} from './fetch-kernel.mjs'
import { KERNEL_TARGETS, KERNEL_VERSION } from './kernel-targets.mjs'

test('parseArgs 支持等号和值分离两种形式', () => {
  const args = parseArgs([
    '--platform',
    'windows',
    '--arch=amd64',
    '--skip-existing',
    '--out',
    'tmp/kernel'
  ])

  assert.deepEqual(args, {
    platform: 'windows',
    arch: 'amd64',
    'skip-existing': true,
    out: 'tmp/kernel'
  })
})

test('resolveRequestedTargets 支持主机目标和全量目标', () => {
  const hostTargets = resolveRequestedTargets({}, { platform: 'win32', arch: 'x64' })
  assert.equal(hostTargets.length, 1)
  assert.equal(hostTargets[0].platform, 'windows')
  assert.equal(hostTargets[0].arch, 'amd64')

  // Число целей задаётся списком, а не переписывается в тесте руками:
  // из-за расхождения этот тест был красным с добавления сборки под Intel Mac.
  const allTargets = resolveRequestedTargets({ all: true }, { platform: 'linux', arch: 'x64' })
  assert.equal(allTargets.length, KERNEL_TARGETS.length)
})

test('buildFilename 根据平台生成正确文件名', () => {
  assert.equal(buildFilename('windows', 'amd64', '1.12.0'), 'sing-box-1.12.0-windows-amd64.zip')
  assert.equal(buildFilename('macos', 'arm64', '1.12.0'), 'sing-box-1.12.0-darwin-arm64.tar.gz')
  assert.equal(buildFilename('linux', 'amd64', '1.12.0'), 'sing-box-1.12.0-linux-amd64.tar.gz')
})

// Ядро в установщике работает у клиента от имени администратора, поэтому берётся
// только у того, кто его выпускает: чужих зеркал в списке быть не должно.
test('ядро качается единственным адресом первоисточника', () => {
  const url = buildDownloadUrl('1.12.0', 'sing-box-1.12.0-windows-amd64.zip')

  assert.equal(
    url,
    'https://github.com/SagerNet/sing-box/releases/download/v1.12.0/sing-box-1.12.0-windows-amd64.zip'
  )
})

test('у каждой цели закреплена контрольная сумма', () => {
  assert.match(KERNEL_VERSION, /^\d+\.\d+\.\d+$/)
  for (const target of KERNEL_TARGETS) {
    assert.match(
      target.sha256 ?? '',
      /^[0-9a-f]{64}$/,
      `${target.platform}/${target.arch} без контрольной суммы`
    )
  }
})
