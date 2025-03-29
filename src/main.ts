import './assets/main.css'
import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { usePinia } from '@/stores'
import { createI18n } from 'vue-i18n'
import zh from './locales/zh.json'
import ru from './locales/ru.json'
import en from './locales/en.json'

const messages = { zh, ru, en }
const savedLocale = localStorage.getItem('locale') || 'en' // или 'ru'

const i18n = createI18n({
  legacy: false,
  locale: savedLocale,
  fallbackLocale: 'en',
  messages,
})


const app = createApp(App)
usePinia(app)
app.use(router)
app.use(i18n) // подключаем i18n
app.mount('#app')
