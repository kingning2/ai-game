/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** 后端 API 地址（构建时注入），默认本机 axum：http://127.0.0.1:17870 */
  readonly VITE_API_BASE_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
