/**
 * Система, на которой мы работаем, влияет на вид окна: на Маке скругления
 * мельче, текст плотнее, а кнопки окна рисует сама система. Класс ставится до
 * сборки темы, поэтому наши размеры доезжают и до готовых элементов naive-ui.
 *
 * Инлайн-скриптом в index.html этого не сделать — его рубит наш CSP.
 */
export const isMacOS = /Mac/i.test(navigator.platform || navigator.userAgent)

if (isMacOS) {
  document.documentElement.classList.add('platform-macos')
}
