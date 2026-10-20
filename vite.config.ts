import tailwindcss from '@tailwindcss/vite';
import vue from '@vitejs/plugin-vue';
import { defineConfig } from 'vite';

// Fester Port für `tauri dev` (devUrl in src-tauri/tauri.conf.json).
export default defineConfig({
    plugins: [vue(), tailwindcss()],
    clearScreen: false,
    server: { port: 1420, strictPort: true, watch: { ignored: ['**/src-tauri/**'] } },
    build: { target: 'es2021' },
});
