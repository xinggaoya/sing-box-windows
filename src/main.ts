import './assets/main.css'

import { createApp } from 'vue'

import App from './App.vue'
import router from './router'
import { usePinia } from '@/stores'
import i18n from './locales'

const app = createApp(App)

usePinia(app)
app.use(router)
app.use(i18n)

app.mount('#app')
