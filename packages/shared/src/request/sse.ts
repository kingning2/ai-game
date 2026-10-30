/**
 * SSE（Server-Sent Events）流式订阅 —— fetch.ts 的流式补充。
 *
 * 适用场景：服务端 → 客户端的单向事件推送（进度、实时战况、播报流）。
 * 帧协议按 SSE 标准解析：空行分帧，`event:` 声明事件名，`data:` 承载数据，
 * `:` 开头的注释行（keep-alive 心跳）忽略。无事件名的帧按约定 JSON.parse
 * 后交给 onMessage（泛型声明业务类型）；带事件名的帧原样转发 onEvent，由调用方释义。
 *
 * 零第三方依赖，错误统一归一化为 ApiError（与 fetch.ts 同口径）。
 * 断线重连策略归调用方（onEnd(reason='error') 后自行决定退避重连）。
 */
import type { QueryParams } from './types';
import { ApiError } from './types';
import { buildQuery, joinUrl } from './utils';

/** 流结束原因：complete=服务端正常关流；aborted=本地取消；error=异常断开（可重连） */
export type SseEndReason = 'complete' | 'aborted' | 'error';

export interface SseStreamOptions {
  /** 查询参数（undefined / null 项丢弃） */
  query?: QueryParams;
  /** 附加请求头 */
  headers?: Record<string, string>;
  /** 取消信号（AbortController.signal）；也可直接用返回的停止函数 */
  signal?: AbortSignal;
}

export interface SseHandlers<T> {
  /** 无事件名的 `data:` 帧：多行 data 以 \n 拼接后 JSON.parse（解析失败按 error 结束） */
  onMessage: (data: T) => void;
  /** 带 `event:` 名的帧原样转发（data 为原文，未解析）；缺省忽略这类帧 */
  onEvent?: (event: string, data: string) => void;
  /** 流结束（恰好回调一次） */
  onEnd: (reason: SseEndReason, error?: ApiError) => void;
}

/** 非 ApiError 异常归一化（与 fetch.ts 同口径；SSE 无超时语义，没有 TimeoutError 分支） */
function normalizeSseError(error: unknown): ApiError {
  if (error instanceof ApiError) return error;
  if (error instanceof DOMException && error.name === 'AbortError') {
    return new ApiError('流已被取消', { kind: 'abort', cause: error });
  }
  if (error instanceof TypeError) {
    return new ApiError('网络错误或请求被阻止', { kind: 'network', cause: error });
  }
  return new ApiError(error instanceof Error ? error.message : '流式请求失败', {
    kind: 'network',
    cause: error instanceof Error ? error : undefined,
  });
}

/** 单帧解析：返回事件名（缺省 null）与拼接后的 data 原文；注释行被丢弃 */
// ponytail: SSE 分帧器是本模块唯一非平凡逻辑，仓库暂无测试基建；
// 引入 vitest 时第一件事就是给它补 「多行 data / 注释帧 / event+data 混合帧」 三条用例
function parseFrame(frame: string): { event: string | null; data: string } {
  let event: string | null = null;
  const data: string[] = [];
  for (const line of frame.split('\n')) {
    if (line.startsWith(':')) continue; // keep-alive 注释帧
    if (line.startsWith('event:')) event = line.slice(6).trim();
    else if (line.startsWith('data:')) data.push(line.slice(5).trimStart());
  }
  return { event, data: data.join('\n') };
}

/**
 * 订阅 SSE 流，返回停止函数（AbortController.abort）。
 *
 * - 响应非 2xx / 无 body：按 error 结束（HTTP 语义归一化在 ApiError.status）
 * - 服务端关流：按 complete 结束（是否代表业务终态由调用方按领域判断，如收到终局帧）
 * - onMessage / onEnd 保证各恰好回调一次；onEnd 后不再有任何回调
 */
export function createSseStream<T>(
  url: string,
  handlers: SseHandlers<T>,
  options: SseStreamOptions = {},
): () => void {
  const controller = new AbortController();
  // 调用方 signal 与内部 abort 信号合一（任一触发都断流）
  const signal =
    options.signal !== undefined
      ? AbortSignal.any([options.signal, controller.signal])
      : controller.signal;
  const settled = { done: false };

  function finish(reason: SseEndReason, error?: ApiError): void {
    if (settled.done) return;
    settled.done = true;
    handlers.onEnd(reason, error);
  }

  void (async () => {
    try {
      const qs = buildQuery(options.query);
      const response = await fetch(joinUrl(undefined, url) + qs, {
        signal,
        headers: { accept: 'text/event-stream', ...options.headers },
      });
      if (!response.ok || response.body === null) {
        throw new ApiError(response.statusText || `SSE HTTP ${response.status}`, {
          kind: 'http',
          status: response.status,
        });
      }
      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let buffer = '';
      for (;;) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        let boundary = buffer.indexOf('\n\n');
        while (boundary !== -1) {
          const frame = parseFrame(buffer.slice(0, boundary));
          buffer = buffer.slice(boundary + 2);
          boundary = buffer.indexOf('\n\n');
          if (frame.event !== null) {
            handlers.onEvent?.(frame.event, frame.data);
            continue;
          }
          if (frame.data.length === 0) continue;
          try {
            handlers.onMessage(JSON.parse(frame.data) as T);
          } catch (parseError) {
            throw new ApiError('SSE 帧不是合法 JSON', {
              kind: 'parse',
              cause: parseError instanceof Error ? parseError : undefined,
            });
          }
        }
      }
      finish('complete');
    } catch (raw) {
      if (signal.aborted) {
        finish('aborted');
        return;
      }
      finish('error', normalizeSseError(raw));
    }
  })();

  return () => controller.abort();
}
