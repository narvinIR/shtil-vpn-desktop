import type { GlobalThemeOverrides } from 'naive-ui'

/**
 * Тема naive-ui собирается из тех же значений, что и наши экраны — из
 * `tokens.css`. Раньше рядом лежал второй список цветов и скруглений
 * (`naive-ui-theme-overrides.json`), и он разъезжался с палитрой молча:
 * кнопка оставалась чужой, когда экран уже был наш.
 *
 * Значения читаются с корня документа, поэтому смена темы (класс `dark`)
 * доезжает и до готовых элементов naive — надо лишь пересобрать объект.
 */

const readVar = (name: string): string => {
  if (typeof document === 'undefined') return ''
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim()
}

/** Кладёт значение в объект, только если переменная объявлена. */
const put = (target: Record<string, string>, key: string, variable: string) => {
  const value = readVar(variable)
  if (value) {
    target[key] = value
  }
}

export const buildNaiveOverrides = (): GlobalThemeOverrides => {
  const common: Record<string, string> = {
    fontSize: readVar('--text-base') || '14px',
    fontWeight: '400',
    fontWeightStrong: '600',
    lineHeight: '1.6',
    heightTiny: '28px',
    heightSmall: '34px',
    heightMedium: '40px',
    heightLarge: '46px',
    heightHuge: '52px',
  }
  put(common, 'fontFamily', '--font-sans')
  // naive-ui красит <body> строкой стиля прямо в элементе, а её не перебить
  // ни одним правилом таблицы. Отсюда и берётся сплошной фон на Маке, если
  // значение не отдать ему самим.
  put(common, 'bodyColor', '--body-bg')
  put(common, 'primaryColor', '--primary-color')
  put(common, 'primaryColorHover', '--primary-hover')
  put(common, 'primaryColorPressed', '--primary-active')
  put(common, 'primaryColorSuppl', '--primary-color')
  put(common, 'infoColor', '--info-color')
  put(common, 'infoColorHover', '--primary-hover')
  put(common, 'infoColorPressed', '--primary-active')
  put(common, 'infoColorSuppl', '--info-color')
  put(common, 'successColor', '--success-color')
  put(common, 'successColorHover', '--success-color')
  put(common, 'successColorPressed', '--success-color')
  put(common, 'successColorSuppl', '--success-color')
  put(common, 'warningColor', '--warning-color')
  put(common, 'warningColorHover', '--warning-color')
  put(common, 'warningColorPressed', '--warning-color')
  put(common, 'warningColorSuppl', '--warning-color')
  put(common, 'errorColor', '--error-color')
  put(common, 'errorColorHover', '--error-color')
  put(common, 'errorColorPressed', '--error-color')
  put(common, 'errorColorSuppl', '--error-color')
  put(common, 'borderRadius', '--radius-md')
  put(common, 'borderRadiusSmall', '--radius-sm')
  put(common, 'boxShadow1', '--shadow-sm')
  put(common, 'boxShadow2', '--shadow-md')
  put(common, 'boxShadow3', '--shadow-lg')

  const button: Record<string, string> = {
    paddingTiny: '0 12px',
    paddingSmall: '0 16px',
    paddingMedium: '0 20px',
    paddingLarge: '0 24px',
    fontWeight: '600',
    fontWeightStrong: '700',
  }
  put(button, 'borderRadius', '--radius-sm')
  put(button, 'borderRadiusSmall', '--radius-xs')
  put(button, 'borderRadiusMedium', '--radius-sm')
  put(button, 'borderRadiusLarge', '--radius-md')

  const card: Record<string, string> = {
    paddingSmall: '16px',
    paddingMedium: '24px',
    paddingLarge: '32px',
    paddingHuge: '40px',
    titleFontWeight: '700',
  }
  put(card, 'borderRadius', '--radius-lg')
  put(card, 'borderColor', '--border-color')

  const input: Record<string, string> = {
    paddingSmall: '0 12px',
    paddingMedium: '0 16px',
    paddingLarge: '0 20px',
    heightSmall: '34px',
    heightMedium: '40px',
    heightLarge: '46px',
  }
  put(input, 'borderRadius', '--radius-sm')
  put(input, 'boxShadowFocus', '--shadow-focus')

  const select: Record<string, string> = {
    paddingSmall: '0 12px',
    paddingMedium: '0 16px',
    paddingLarge: '0 20px',
    heightSmall: '34px',
    heightMedium: '40px',
    heightLarge: '46px',
  }
  put(select, 'borderRadius', '--radius-sm')

  const tag: Record<string, string> = {
    paddingMedium: '0 10px',
    heightMedium: '26px',
    fontWeight: '600',
  }
  put(tag, 'borderRadius', '--radius-xs')

  const modal: Record<string, string> = {
    paddingMedium: '28px',
    titleFontWeight: '700',
  }
  put(modal, 'borderRadius', '--radius-2xl')
  put(modal, 'boxShadow', '--shadow-xl')

  const tooltip: Record<string, string> = { padding: '8px 12px' }
  put(tooltip, 'borderRadius', '--radius-xs')
  put(tooltip, 'boxShadow', '--shadow-md')

  const popover: Record<string, string> = { padding: '16px' }
  put(popover, 'borderRadius', '--radius-md')
  put(popover, 'boxShadow', '--shadow-lg')

  const dropdown: Record<string, string> = { padding: '6px', optionHeightMedium: '36px' }
  put(dropdown, 'borderRadius', '--radius-md')
  put(dropdown, 'boxShadow', '--shadow-lg')

  const switchStyle: Record<string, string> = {}
  put(switchStyle, 'railColorActive', '--primary-color')
  put(switchStyle, 'railColor', '--border-strong')

  return {
    common,
    Button: button,
    Card: card,
    Input: input,
    Select: select,
    Switch: switchStyle,
    Tag: tag,
    Modal: modal,
    Dialog: modal,
    Tooltip: tooltip,
    Popover: popover,
    Dropdown: dropdown,
    Layout: { headerHeight: readVar('--header-height') || '48px' },
  } as GlobalThemeOverrides
}
