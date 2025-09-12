import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';
import Home from '../pages/home.vue';
import Record from '../pages/record.vue';

const routes: RouteRecordRaw[] = [
    { path: '/', component: Home },
    { path: '/record', component: Record },
]

export const router = createRouter({
    history: createWebHistory(),
    routes,
})