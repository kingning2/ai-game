import { Method } from './fetch';
import {
  ApiError,
  SUCCESS_CODE,
  type ApiResponse,
  type JsonValue,
  type QueryParams,
  type RequestOptions,
} from './types';

const IDEMPOTENT_METHODS = new Set(['GET', 'HEAD']);

/** 幂等方法（GET/HEAD）：重复发起语义安全，默认开启去重 */
export function isIdempotentMethod(method: string): boolean {
  return IDEMPOTENT_METHODS.has(method.toUpperCase());
}

/** 拼接 baseURL 与 url（url 为 http(s) 绝对地址时原样返回） */
export function joinUrl(baseURL: string | undefined, url: string): string {
  if (!baseURL || /^https?:\/\//i.test(url)) return url;
  return `${baseURL.replace(/\/+$/, '')}/${url.replace(/^\/+/, '')}`;
}

/** 序列化查询参数，丢弃 undefined / null 项 */
export function buildQuery(query: QueryParams | undefined): string {
  if (!query) return '';
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined || value === null) continue;
    params.append(key, String(value));
  }
  const qs = params.toString();
  return qs ? `?${qs}` : '';
}

/** 去重键：method + url + 排序后的 query（key 顺序不同视为同一请求） */
export function dedupeKey(method: Method, url: string, query: QueryParams | undefined): string {
  const qs = query
    ? Object.keys(query)
        .filter((k) => query[k] !== undefined && query[k] !== null)
        .sort()
        .map((k) => `${k}=${String(query[k])}`)
        .join('&')
    : '';
  return `${method.toUpperCase()} ${url}${qs ? `?${qs}` : ''}`;
}

/** 合并调用方 signal 与超时 signal，任一触发都取消请求 */
export function combineSignals(
  signal: AbortSignal | undefined,
  timeoutMs: number | undefined,
): AbortSignal | undefined {
  const timeoutSignal = timeoutMs && timeoutMs > 0 ? AbortSignal.timeout(timeoutMs) : undefined;
  if (signal && timeoutSignal) return AbortSignal.any([signal, timeoutSignal]);
  return signal ?? timeoutSignal;
}

/** 判断响应体是否是约定的 {code, message, data} 包裹（data 允许为任意值/缺失内容，但字段需在） */
export function isEnvelope(value: object | null | undefined): value is ApiResponse {
  return (
    typeof value === 'object' &&
    value !== null &&
    typeof (value as ApiResponse).code === 'number' &&
    typeof (value as ApiResponse).message === 'string' &&
    'data' in value
  );
}

/** 不是包裹结构时抛 parse 错误；调用处已确认是包裹时由泛型声明 data 的类型 */
export function assertEnvelope<T = JsonValue>(
  value: object | null | undefined,
  status: number,
): ApiResponse<T> {
  if (isEnvelope(value)) return value as ApiResponse<T>;
  throw new ApiError(`响应不是约定的 {code, message, data} 结构（HTTP ${status}）`, {
    kind: 'parse',
    status,
  });
}

/** 校验业务码并解包 data —— 两个适配器的响应管道最终都走这里 */
export function unwrapEnvelope<T>(
  envelope: ApiResponse<T>,
  successCode = SUCCESS_CODE,
  status?: number,
): T {
  if (envelope.code !== successCode) {
    throw new ApiError(envelope.message || `业务错误 code=${envelope.code}`, {
      kind: 'business',
      code: envelope.code,
      status,
    });
  }
  return envelope.data;
}

/**
 * 进行中请求登记表：相同幂等请求只发一次，后来的调用共享同一个 Promise。
 * Promise 结束（成功或失败）后移除登记。
 */
export function createDedupeCache() {
  // 值类型 Promise<never>：never 可赋给任何 Promise<T>（读方向免费），写方向由调用点断言收窄
  const inflight = new Map<string, Promise<never>>();
  return {
    run<T>(key: string, enabled: boolean, factory: () => Promise<T>): Promise<T> {
      if (!enabled) return factory();
      const cached = inflight.get(key);
      if (cached) return cached;
      const promise = factory().finally(() => inflight.delete(key)) as Promise<never>;
      inflight.set(key, promise);
      return promise;
    },
  };
}

/** 计算本次请求是否启用去重 */
export function shouldDedupe(method: string, options: RequestOptions): boolean {
  return options.dedupe ?? isIdempotentMethod(method);
}
