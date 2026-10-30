/**
 * 请求层公共类型与约定。
 *
 * 后端统一响应包裹为 { code, message, data }，code === 成功码(默认 0) 时 data 为业务数据，
 * 否则请求层抛出 ApiError(kind: 'business')。
 */

/** 业务成功码 */
export const SUCCESS_CODE = 0;

/**
 * 递归 JSON 值：替代 any 的「未知数据」表示（项目规范：禁 unknown，未知结构用具体类型或泛型）。
 * 仅作泛型默认值用——调用方永远应显式声明业务类型。
 */
export type JsonValue =
  | string
  | number
  | boolean
  | null
  | JsonValue[]
  | { [key: string]: JsonValue };

/** 后端统一响应包裹（data 允许为任意业务数据，由调用方泛型声明） */
export interface ApiResponse<T = JsonValue> {
  code: number;
  message: string;
  data: T;
}

/** 错误类别 */
export type ApiErrorKind =
  | 'business' // HTTP 2xx 但业务 code 非成功码
  | 'http' // 非 2xx 且响应体不是约定包裹（网关层错误等）
  | 'network' // 网络 / DNS / CORS 层失败
  | 'abort' // 手动取消或超时
  | 'parse'; // 响应不是合法 JSON 或不符合包裹结构

/** 统一错误类型：axios / fetch 两个适配器抛出的都是它 */
export class ApiError extends Error {
  override name = 'ApiError';
  /** 错误类别 */
  readonly kind: ApiErrorKind;
  /** 业务码：business 时为后端 code，其余为 -1 */
  readonly code: number;
  /** HTTP 状态码（拿得到就带） */
  readonly status?: number;

  constructor(
    message: string,
    options: { kind: ApiErrorKind; code?: number; status?: number; cause?: object },
  ) {
    super(message, options.cause !== undefined ? { cause: options.cause } : undefined);
    this.kind = options.kind;
    this.code = options.code ?? -1;
    this.status = options.status;
  }
}

/** 查询参数：undefined / null 的项会被丢弃 */
export type QueryParams = Record<string, string | number | boolean | undefined | null>;

export interface RequestOptions {
  /** 附加请求头 */
  headers?: Record<string, string>;
  /** 查询参数 */
  query?: QueryParams;
  /** 超时毫秒数，超时按 abort 处理 */
  timeout?: number;
  /** 取消信号：传 AbortController.signal 即可取消请求 */
  signal?: AbortSignal;
  /**
   * 幂等去重：进行中的相同请求（method + url + query 相同）只发一次，共享同一个 Promise。
   * 默认幂等方法（GET/HEAD）开启，其余关闭。
   */
  dedupe?: boolean;
}

/** 两个适配器共同的方法签名，可互相替换 */
export interface RequestClient {
  get<T = JsonValue>(url: string, options?: RequestOptions): Promise<T>;
  post<T = JsonValue>(url: string, body?: object, options?: RequestOptions): Promise<T>;
  put<T = JsonValue>(url: string, body?: object, options?: RequestOptions): Promise<T>;
  patch<T = JsonValue>(url: string, body?: object, options?: RequestOptions): Promise<T>;
  delete<T = JsonValue>(url: string, options?: RequestOptions): Promise<T>;
}

/** 客户端公共配置 */
export interface RequestClientConfig {
  /** 基础地址，与 url 拼接；url 为绝对地址时不拼接 */
  baseURL?: string;
  /** 默认超时毫秒数（可被 RequestOptions.timeout 覆盖） */
  timeout?: number;
  /** 客户端级请求头（可被 RequestOptions.headers 覆盖） */
  headers?: Record<string, string>;
  /** 业务成功码，默认 0 */
  successCode?: number;
}
