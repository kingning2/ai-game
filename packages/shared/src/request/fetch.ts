import type { ApiResponse, RequestClient, RequestClientConfig, RequestOptions } from './types';
import { ApiError, SUCCESS_CODE } from './types';
import {
  assertEnvelope,
  buildQuery,
  combineSignals,
  createDedupeCache,
  dedupeKey,
  isEnvelope,
  joinUrl,
  shouldDedupe,
  unwrapEnvelope,
} from './utils';

/** 请求拦截器上下文：可修改 url 与 RequestInit（headers/signal 等） */
export interface FetchRequestContext {
  url: string;
  init: RequestInit;
}

export type Method = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

export interface FetchInterceptors {
  /** 请求拦截器 */
  request?: (ctx: FetchRequestContext) => FetchRequestContext | Promise<FetchRequestContext>;
  /**
   * 响应拦截器：拿到解析后的统一包裹与原始 Response，可转换 / 打点，
   * 返回的包裹继续走业务码校验与解包。
   */
  response?: <D>(
    envelope: ApiResponse<D>,
    response: Response,
  ) => ApiResponse<D> | Promise<ApiResponse<D>>;
}

export interface FetchClientConfig extends RequestClientConfig {
  interceptors?: FetchInterceptors;
}

/** 基于原生 fetch 的请求客户端（零第三方依赖） */
export function createFetchClient(config: FetchClientConfig = {}): RequestClient {
  const {
    baseURL,
    timeout,
    headers: baseHeaders,
    successCode = SUCCESS_CODE,
    interceptors,
  } = config;
  const dedupe = createDedupeCache();

  /** 非 2xx：响应体若仍是约定包裹，按业务错误还原，否则按 http 错误 */
  async function toHttpError(response: Response): Promise<ApiError> {
    const raw: object | undefined = await response.json().catch(() => undefined);
    if (isEnvelope(raw)) {
      return new ApiError(raw.message || `HTTP ${response.status}`, {
        kind: 'business',
        code: raw.code,
        status: response.status,
      });
    }
    return new ApiError(response.statusText || `HTTP ${response.status}`, {
      kind: 'http',
      status: response.status,
    });
  }

  async function parseJson(response: Response): Promise<object> {
    try {
      return (await response.json()) as object;
    } catch (error) {
      throw new ApiError('响应不是合法 JSON', {
        kind: 'parse',
        status: response.status,
        cause: error,
      });
    }
  }

  /** 归一化抛出的非 ApiError 异常（网络 / 取消 / 其余未知异常都包成 ApiError） */
  function normalize(error: object): ApiError {
    if (error instanceof ApiError) return error;
    if (
      error instanceof DOMException &&
      (error.name === 'AbortError' || error.name === 'TimeoutError')
    ) {
      return new ApiError('请求已取消或超时', { kind: 'abort', cause: error });
    }
    if (error instanceof TypeError) {
      return new ApiError('网络错误或请求被阻止', { kind: 'network', cause: error });
    }
    return new ApiError(error instanceof Error ? error.message : '请求失败', {
      kind: 'network',
      cause: error,
    });
  }

  async function request<T>(
    method: Method,
    url: string,
    body: object | undefined,
    options: RequestOptions,
  ): Promise<T> {
    let ctx: FetchRequestContext = {
      url: joinUrl(baseURL, url) + buildQuery(options.query),
      init: {
        method,
        headers: { ...baseHeaders, ...options.headers },
        // 手动取消与超时统一合并成一个 signal，任一触发都取消请求
        signal: combineSignals(options.signal, options.timeout ?? timeout),
      },
    };
    if (body !== undefined) {
      ctx.init.body = JSON.stringify(body);
      ctx.init.headers = {
        'Content-Type': 'application/json',
        ...(ctx.init.headers as Record<string, string>),
      };
    }

    // 请求拦截器
    if (interceptors?.request) ctx = await interceptors.request(ctx);

    // 响应管道：网络 → HTTP 状态 → 解析包裹 → 响应拦截器 → 业务码校验解包
    try {
      const response = await fetch(ctx.url, ctx.init);
      if (!response.ok) throw await toHttpError(response);
      let envelope = assertEnvelope<T>(await parseJson(response), response.status);
      if (interceptors?.response) envelope = await interceptors.response(envelope, response);
      return unwrapEnvelope<T>(envelope, successCode, response.status);
    } catch (error) {
      throw normalize(error);
    }
  }

  const run = <T>(
    method: Method,
    url: string,
    body: object | undefined,
    options: RequestOptions = {},
  ) =>
    dedupe.run(dedupeKey(method, url, options.query), shouldDedupe(method, options), () =>
      request<T>(method, url, body, options),
    );

  return {
    get: (url, options) => run('GET', url, undefined, options),
    post: (url, body, options) => run('POST', url, body, options),
    put: (url, body, options) => run('PUT', url, body, options),
    patch: (url, body, options) => run('PATCH', url, body, options),
    delete: (url, options) => run('DELETE', url, undefined, options),
  };
}
