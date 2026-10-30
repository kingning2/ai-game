/**
 * web → server HTTP 通道唯一出口：全站仅此一个 createFetchClient 实例，export 为 `http`。
 * 端点函数一律 `import { http } from '../apis/http'` 组装（见 ./werewolf.ts），页面禁止裸 fetch。
 * 故障分类（ApiError → ServerFailure）也收口在本文件，UI 层只消费 ServerFailure。
 * server 端点见 rust/server/src/routes/werewolf/mod.rs
 */
import { ApiError, createFetchClient } from '@agent-packages/shared';

export type { ApiError };

export const http = createFetchClient({ baseURL: '/api', timeout: 8000 });

/**
 * server 业务错误码枚举 —— rust/server/src/error/app_error.rs::AppError::code 的前端镜像。
 * 后端 code/status 只在这一处维护；两端新增/调整错误码必须同步改这里。
 * （本仓库规范优先联合字面量，用 const 对象 + keyof 类型代替 TS enum，语义等价且可树摇）
 */
export const SERVER_ERROR_CODE = {
  /** 400：参数取值/语义非法 */
  badRequest: 10001,
  /** 401：未登录 / 凭证缺失或无效 */
  unauthorized: 10002,
  /** 403：已登录但无权限 */
  forbidden: 10003,
  /** 404：资源不存在（房间不存在 / 没有进行中的对局） */
  notFound: 10004,
  /** 409：资源状态冲突（重复开局等） */
  conflict: 10005,
  /** 405：HTTP 方法不支持 */
  methodNotAllowed: 10006,
  /** 413：请求体超出大小限制 */
  payloadTooLarge: 10007,
  /** 415：Content-Type 不支持 */
  unsupportedMediaType: 10008,
  /** 422：参数结构无法解析 */
  unprocessable: 10009,
  /** 429：触发限流 */
  tooManyRequests: 10010,
  /** 408：请求处理超时 */
  timeout: 10011,
  /** 500：server 内部错误（含模型凭证未配置等，详情只在 server 日志） */
  internal: 50000,
  /** 503：依赖服务不可用（上游模型网络层失败 / 上游 5xx） */
  serviceUnavailable: 50001,
} as const;

export type ServerErrorCode = (typeof SERVER_ERROR_CODE)[keyof typeof SERVER_ERROR_CODE];

/**
 * 页面可见的 server 故障分类：kind 对齐 shared ApiError 的 ApiErrorKind，
 * business 时 code 为后端业务码（SERVER_ERROR_CODE）。
 * UI 文案按这个联合逐分支给具体提示，禁止把原始错误透传给用户。
 */
export type ServerFailure =
  | { kind: 'business'; code: number; message: string }
  | { kind: 'http'; status: number; message: string }
  | { kind: 'network' }
  | { kind: 'abort' }
  | { kind: 'parse'; message: string }
  | { kind: 'unknown'; message: string };

/** 任意抛出值 → ServerFailure（ApiError 之外只有兜底分支，不会漏接） */
export function toServerFailure(error: unknown): ServerFailure {
  if (error instanceof ApiError) {
    switch (error.kind) {
      case 'business':
        return { kind: 'business', code: error.code, message: error.message };
      case 'http':
        return { kind: 'http', status: error.status ?? 0, message: error.message };
      case 'network':
        return { kind: 'network' };
      case 'abort':
        return { kind: 'abort' };
      case 'parse':
        return { kind: 'parse', message: error.message };
    }
  }
  if (error instanceof Error) return { kind: 'unknown', message: error.message };
  return { kind: 'unknown', message: String(error) };
}
