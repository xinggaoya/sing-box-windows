import './assets/main.css'

import { createApp } from 'vue'

import App from './App.vue'
import router from './router'
import { usePinia } from '@/stores'

const app = createApp(App)

usePinia(app)
app.use(router)

app.mount('#app')
