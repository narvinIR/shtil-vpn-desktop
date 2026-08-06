import { createRouter, createWebHashHistory } from 'vue-router'
import Layout from '@/components/layout/MainLayout.vue'

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    // Пустой экран для режима трея: свёрнутое окно не держит интерфейс в памяти
    {
      path: '/blank',
      name: 'Blank',
      component: () => import('@/views/BlankView.vue'),
      meta: {
        isBlankPage: true,
      },
    },
    // Главная раскладка: шапка, боковое меню, экраны
    {
      path: '/',
      name: 'index',
      component: Layout,
      children: [
        {
          path: '/',
          name: 'Home',
          component: () => import('@/views/HomeView.vue'),
        },
        {
          path: '/sub',
          name: 'Key',
          component: () => import('@/views/KeyView.vue'),
        },
        {
          path: '/log',
          name: 'Diagnostics',
          component: () => import('@/views/DiagnosticsView.vue'),
        },
        {
          path: '/setting',
          name: 'Setting',
          component: () => import('@/views/SettingView.vue'),
        },
      ],
    },
    // Трей помнит последний открытый экран. Экраны форка удалены, и без этого
    // правила запомненный адрес открывал бы пустое окно вместо приложения.
    {
      path: '/:pathMatch(.*)*',
      redirect: '/',
    },
  ],
})

export default router
