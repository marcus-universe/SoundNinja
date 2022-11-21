import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import store from './store'
import JsonHandle from './functions/JsonHandle'

createApp(App).use(store).use(router).use(JsonHandle).mount('#app')
