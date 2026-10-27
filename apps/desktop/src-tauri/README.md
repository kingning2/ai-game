# src-tauri（桌面壳）

纯 Tauri 2 壳，**不含任何前端代码**——装载的是 `apps/web` 的构建产物。

## 职责（仅此三件）

1. **拉起 sidecar**：`setup` 里 `app.shell().sidecar("server").spawn()`，对应 `tauri.conf.json` 的 `bundle.externalBin: binaries/server`（由 `scripts/build-sidecar.mjs` 从 `rust/server` 构建复制，命名 `server-x86_64-pc-windows-msvc.exe`）
2. **转发 sidecar 日志**：sidecar 的 stdout/stderr 打到终端（`[server] ...` 前缀），方便排查
3. **退出回收**：`ServerChild(Mutex<Option<CommandChild>>)` 持有子进程，`RunEvent::ExitRequested` 时 kill，防止残留进程占用 17870 端口

## 前后端通信约定

**不使用 Tauri IPC（invoke）**。前端通过 HTTP 直连 sidecar（`http://127.0.0.1:17870`，即 `apps/web/src/App.tsx` 的 `VITE_API_BASE_URL` 默认值）。

## 开发 / 构建

```bash
pnpm dev:win    # 开发：beforeDevCommand 会先 build sidecar + 启动 web dev server(5173)，壳加载 devUrl
pnpm build:win  # 打包：--release sidecar + web build → frontendDist ../../web/dist
```

⚠️ 跑桌面壳前先停掉 `pnpm server`（端口冲突）。
