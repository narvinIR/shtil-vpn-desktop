import { useMessage } from 'naive-ui'
import type { MessageApi } from 'naive-ui'

/**
 * Сообщения приложения. Отличие от naive-ui одно: ошибка живёт дольше
 * остального.
 *
 * Успех сообщает то, что и так видно на экране, — три секунды по умолчанию ему
 * хватает. Ошибку человек читает: там имя того, что не получилось, и решение,
 * что делать дальше. С общей длительностью она гасла раньше, чем дочитывали.
 * Закрытие рукой и ожидание под курсором задаёт провайдер в `App.vue`.
 */
const ERROR_DURATION_MS = 8000

export const useAppMessage = (): MessageApi => {
  const message = useMessage()

  return {
    ...message,
    error: (content, options) => message.error(content, { duration: ERROR_DURATION_MS, ...options }),
  }
}
