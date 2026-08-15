import { reactive, ref, watch } from 'vue'

interface MessageApiLike {
  success: (content: string) => void
  error: (content: string) => void
}

interface AppStoreLike {
  isDataRestored: boolean
  systemProxyBypass: string
  tunMtu: number
  tunStack: string
  tunEnableIpv6: boolean
  tunRouteExcludeAddress: string[] | null
  tunAutoRoute: boolean
  tunStrictRoute: boolean
  tunSelfHealEnabled: boolean
  tunSelfHealCooldownSecs: number
  saveToBackend: (options?: { applyRuntime?: boolean }) => Promise<void>
}

interface UseAdvancedSettingsFormOptions {
  appStore: AppStoreLike
  message: MessageApiLike
  t: (key: string, params?: Record<string, string | number>) => string
}

const IPV4_CIDR_RE =
  /^(25[0-5]|2[0-4]\d|1?\d?\d)(\.(25[0-5]|2[0-4]\d|1?\d?\d)){3}\/([0-9]|[12]\d|3[0-2])$/
const IPV6_CIDR_RE = /^[0-9A-Fa-f:]+\/([0-9]|[1-9]\d|1[01]\d|12[0-8])$/

const isLikelyCidr = (value: string, family: 'ipv4' | 'ipv6') => {
  const trimmed = value.trim()
  if (!trimmed) return false
  return family === 'ipv4' ? IPV4_CIDR_RE.test(trimmed) : IPV6_CIDR_RE.test(trimmed)
}

const isLikelyAnyCidr = (value: string) =>
  isLikelyCidr(value, 'ipv4') || isLikelyCidr(value, 'ipv6')

const parseTunRouteExcludeAddressLines = (value: string) =>
  value
    .split(/\r?\n/)
    .map((line, index) => ({
      line: index + 1,
      value: line.trim(),
    }))
    .filter((entry) => entry.value.length > 0)

const normalizeTunRouteExcludeAddress = (value: string) => {
  const lines = parseTunRouteExcludeAddressLines(value).map((entry) => entry.value)
  return lines.length > 0 ? lines : null
}

export const useAdvancedSettingsForm = (options: UseAdvancedSettingsFormOptions) => {
  // Начальные значения берём из хранилища, а не пишем числами здесь: свои
  // умолчания в форме разъезжались с настоящими (размер пакета стоял 9000 при
  // рабочих 1500), и человек, нажавший «Сохранить» раньше, чем настройки
  // успевали подтянуться, тихо портил себе связь.
  const savingAdvanced = ref(false)
  const proxyAdvancedForm = reactive({
    systemProxyBypass: options.appStore.systemProxyBypass,
    tunMtu: options.appStore.tunMtu,
    tunStack: options.appStore.tunStack as 'system' | 'gvisor' | 'mixed',
    tunEnableIpv6: options.appStore.tunEnableIpv6,
    tunRouteExcludeAddressText: options.appStore.tunRouteExcludeAddress?.join('\n') ?? '',
    tunAutoRoute: options.appStore.tunAutoRoute,
    tunStrictRoute: options.appStore.tunStrictRoute,
    tunSelfHealEnabled: options.appStore.tunSelfHealEnabled,
    tunSelfHealCooldownSecs: options.appStore.tunSelfHealCooldownSecs,
  })

  watch(
    () => options.appStore.isDataRestored,
    (restored) => {
      if (!restored) return

      proxyAdvancedForm.systemProxyBypass = options.appStore.systemProxyBypass
      proxyAdvancedForm.tunMtu = options.appStore.tunMtu
      proxyAdvancedForm.tunStack = options.appStore.tunStack as 'system' | 'gvisor' | 'mixed'
      proxyAdvancedForm.tunEnableIpv6 = options.appStore.tunEnableIpv6
      proxyAdvancedForm.tunRouteExcludeAddressText =
        options.appStore.tunRouteExcludeAddress?.join('\n') ?? ''
      proxyAdvancedForm.tunAutoRoute = options.appStore.tunAutoRoute
      proxyAdvancedForm.tunStrictRoute = options.appStore.tunStrictRoute
      proxyAdvancedForm.tunSelfHealEnabled = options.appStore.tunSelfHealEnabled
      proxyAdvancedForm.tunSelfHealCooldownSecs = options.appStore.tunSelfHealCooldownSecs

    },
    { immediate: true },
  )

  const saveProxyAdvancedSettings = async () => {
    // Размер пакета вне разумных границ рвёт связь молча: туннель поднимается,
    // а страницы не открываются. Фраза для этого в словаре была, проверки — нет.
    if (proxyAdvancedForm.tunMtu < 576 || proxyAdvancedForm.tunMtu > 9000) {
      options.message.error(options.t('setting.proxyAdvanced.errors.invalidMtu'))
      return
    }

    if (
      proxyAdvancedForm.tunSelfHealEnabled &&
      (proxyAdvancedForm.tunSelfHealCooldownSecs < 15 || proxyAdvancedForm.tunSelfHealCooldownSecs > 600)
    ) {
      options.message.error(options.t('setting.proxyAdvanced.errors.selfHealCooldownInvalid'))
      return
    }

    const invalidRouteExcludeAddressLine = parseTunRouteExcludeAddressLines(
      proxyAdvancedForm.tunRouteExcludeAddressText,
    ).find((entry) => !isLikelyAnyCidr(entry.value))

    if (invalidRouteExcludeAddressLine) {
      options.message.error(
        options.t('setting.proxyAdvanced.errors.tunRouteExcludeAddressInvalidLine', {
          line: invalidRouteExcludeAddressLine.line,
          value: invalidRouteExcludeAddressLine.value,
        }),
      )
      return
    }

    savingAdvanced.value = true
    try {
      options.appStore.systemProxyBypass = proxyAdvancedForm.systemProxyBypass
      options.appStore.tunMtu = proxyAdvancedForm.tunMtu
      options.appStore.tunAutoRoute = proxyAdvancedForm.tunAutoRoute
      options.appStore.tunStrictRoute = proxyAdvancedForm.tunStrictRoute
      options.appStore.tunStack = proxyAdvancedForm.tunStack
      options.appStore.tunEnableIpv6 = proxyAdvancedForm.tunEnableIpv6
      options.appStore.tunRouteExcludeAddress = normalizeTunRouteExcludeAddress(
        proxyAdvancedForm.tunRouteExcludeAddressText,
      )
      options.appStore.tunSelfHealEnabled = proxyAdvancedForm.tunSelfHealEnabled
      options.appStore.tunSelfHealCooldownSecs = proxyAdvancedForm.tunSelfHealCooldownSecs

      await options.appStore.saveToBackend({ applyRuntime: true })
      options.message.success(options.t('common.saveSuccess'))
    } catch {
      options.message.error(options.t('common.saveFailed'))
    } finally {
      savingAdvanced.value = false
    }
  }

  return {
    savingAdvanced,
    proxyAdvancedForm,
    saveProxyAdvancedSettings,
  }
}
