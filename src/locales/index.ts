import { createI18n } from 'vue-i18n'
import enUS from './en-US'
import ruRU from './ru-RU'
import deDE from './de-DE'
import esES from './es-ES'
import faIR from './fa-IR'

// Языки продукта — те же пять, что в боте и в приложении для телефона: русский,
// английский, немецкий, испанский, персидский. Китайский и японский достались от
// форка и убраны: чужой набор языков в списке выбора выдаёт, что программа не наша.
export type LocaleCode = 'ru-RU' | 'en-US' | 'de-DE' | 'es-ES' | 'fa-IR'

export interface SupportedLocale {
  code: LocaleCode
  name: string
}

export const DEFAULT_LOCALE: LocaleCode = 'ru-RU'

export const supportedLocales: SupportedLocale[] = [
  { code: 'ru-RU', name: 'Русский' },
  { code: 'en-US', name: 'English' },
  { code: 'de-DE', name: 'Deutsch' },
  { code: 'es-ES', name: 'Español' },
  { code: 'fa-IR', name: 'فارسی' },
]

const i18n = createI18n({
  legacy: false,
  locale: DEFAULT_LOCALE,
  // Запасной язык — английский, а НЕ язык по умолчанию: раньше русский был
  // запасным сам себе, и недостающий ключ выходил на экран сырым именем
  // («home.statusDescriptions.runningDesc» на главной). Три новых словаря
  // намеренно покрывают только наши экраны — глубокие настройки форка честно
  // доезжают по-английски, а не половинчатым переводом.
  fallbackLocale: ['en-US', DEFAULT_LOCALE],
  messages: {
    'ru-RU': ruRU,
    'en-US': enUS,
    'de-DE': deDE,
    'es-ES': esES,
    'fa-IR': faIR,
  },
  globalInjection: true,
})

export default i18n
