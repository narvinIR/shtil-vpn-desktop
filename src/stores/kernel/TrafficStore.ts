import { defineStore } from 'pinia'
import { ref } from 'vue'
import { APP_EVENTS } from '@/constants/events'
import type { TrafficDataPayload } from '@/types/events'
import { eventService } from '@/services/event-service'

// 连接状态接口
interface ConnectionState {
  connected: boolean
  connecting: boolean
  error: Error | null
}

export const useTrafficStore = defineStore(
  'traffic',
  () => {
    // 流量数据
    const traffic = ref({
      up: 0,
      down: 0,
      totalUp: 0,
      totalDown: 0,
    })

    // 连接状态
    const connectionState = ref<ConnectionState>({
      connected: false,
      connecting: false,
      error: null,
    })

    // 事件监听器状态
    let eventListenersSetup = false

    // 更新流量统计数据
    const updateTrafficStats = (data: TrafficDataPayload) => {
      if (data && 'up' in data && 'down' in data) {
        try {
          // Ядро присылает СКОРОСТЬ за последнюю секунду, а не накопленный
          // счётчик. Здесь бралась разность двух скоростей — при ровной закачке
          // она около нуля, и «за всё время» показывало почти ничего.
          const currentUp = Number(data.up) || 0
          const currentDown = Number(data.down) || 0

          traffic.value = {
            up: currentUp,
            down: currentDown,
            totalUp: (traffic.value.totalUp || 0) + currentUp,
            totalDown: (traffic.value.totalDown || 0) + currentDown,
          }

          // 如果数据接收正常，但当前状态不是连接状态，更新状态
          if (!connectionState.value.connected) {
            connectionState.value.connected = true
            connectionState.value.connecting = false
            connectionState.value.error = null
          }
        } catch (error) {
          console.error('处理流量数据时出错:', error, data)
        }
      }
    }

    // 设置Tauri事件监听器
    const setupEventListeners = async () => {
      if (eventListenersSetup) return

      try {
        // 监听流量数据事件
        await eventService.onTrafficData((data) => {
          if (data && typeof data === 'object' && 'up' in data && 'down' in data) {
            updateTrafficStats(data as TrafficDataPayload)
          }
        })

        // 当收到流量数据时，说明连接正常
        connectionState.value.connected = true
        connectionState.value.connecting = false
        connectionState.value.error = null

        eventListenersSetup = true
      } catch (error) {
        console.error('❌ 流量Store事件监听器设置失败:', error)
      }
    }

    // 清理事件监听器
    const cleanupEventListeners = () => {
      if (!eventListenersSetup) return

      try {
        eventService.removeEventListener(APP_EVENTS.trafficData)
      } catch (error) {
        console.error('清理流量监听器时出错:', error)
      } finally {
        eventListenersSetup = false
      }
    }

    // 重置流量统计
    const resetStats = () => {
      traffic.value = {
        up: 0,
        down: 0,
        totalUp: 0,
        totalDown: 0,
      }
      connectionState.value = {
        connected: false,
        connecting: false,
        error: null,
      }
    }

    // 初始化Store
    const initializeStore = async () => {
      try {
        await setupEventListeners()
      } catch (error) {
        console.error('❌ TrafficStore 初始化失败:', error)
      }
    }

    return {
      traffic,
      connectionState,
      setupEventListeners,
      cleanupEventListeners,
      resetStats,
      updateTrafficStats,
      initializeStore, // 添加这个方法
    }
  },
)
