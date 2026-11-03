import { resolve } from 'node:path';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

// https://vite.dev/config/
export default defineConfig(() => ({
  // React Compiler：oxc 原生 Rust 实现（oxc-transform-react），零 Babel
  plugins: [react({ compiler: true }), tailwindcss()],

  resolve: {
    alias: [
      // workspace 包源码直连：exports 指向 dist 会被 optimizeDeps 预构建缓存，
      // 改包 src 后页面不生效；精确匹配主入口（不拦 styles.css 子路径，它本就直出源文件）
      {
        find: /^@agent-packages\/shared-ui$/,
        replacement: resolve(import.meta.dirname, '../../packages/shared-ui/src/index.ts'),
      },
      {
        find: /^@agent-packages\/shared$/,
        replacement: resolve(import.meta.dirname, '../../packages/shared/index.ts'),
      },
      {
        find: /^@agent-packages\/game$/,
        replacement: resolve(import.meta.dirname, '../../packages/game/src/index.ts'),
      },
      {
        find: /^@agent-packages\/animation$/,
        replacement: resolve(import.meta.dirname, '../../packages/animation/src/index.ts'),
      },
    ],
  },

  server: {
    // desktop 壳的 tauri.conf.json 里 devUrl 固定指向这个端口；
    // host 0.0.0.0 同时绑 v4/v6（默认只绑 ::1，proxy 回环 IPv4 会 ECONNREFUSED）
    port: 5173,
    strictPort: true,
    host: '0.0.0.0',
    proxy: {
      // Rust 伴生服务（对局控制面 / TTS 语音合成），前端同源走相对路径
      // ws:true：录制推流用 WebSocket 升级请求（对象形式才支持，字符串简写不转发 ws）
      '/api': { target: 'http://127.0.0.1:17870', ws: true },
    },
    watch: {
      // 包源码在 vite root（apps/web）之外，默认不监听；不追加的话改包源码
      // Tailwind 不会重扫、页面也不 HMR（得重启 dev server 才生效）
      ignored: [
        '**/.git/**',
        '**/node_modules/**',
        '!**/packages/shared/src/**',
        '!**/packages/shared-ui/src/**',
      ],
    },
  },
}));
