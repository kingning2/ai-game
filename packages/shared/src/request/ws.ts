/**
 * WebSocket 二进制推流 —— sse.ts 的对向补充（客户端 → 服务端）。
 *
 * 适用场景：客户端持续向服务端喂数据（录像推流、麦克风采集等），服务端按连接生命周期
 * 处理（如转交 ffmpeg 落盘）。控制回执走文本帧（JSON 字符串），数据走二进制帧。
 *
 * 零第三方依赖；URL 由 joinUrl/buildQuery 复用（http(s) → ws(s) 自动换协议）。
 * 断线重连归调用方（推流是单向有状态会话，静默重连会丢段，不该在底层假装无损）。
 */
import type { QueryParams } from './types';
import { buildQuery, joinUrl } from './utils';

/** http(s) → ws(s)：同源相对路径直接用 location.host */
function toWsUrl(
  baseURL: string | undefined,
  path: string,
  query: QueryParams | undefined,
): string {
  const httpUrl = joinUrl(baseURL, path) + buildQuery(query);
  if (httpUrl.startsWith('//'))
    return `${location.protocol === 'https:' ? 'wss' : 'ws'}:${httpUrl}`;
  return httpUrl
    .replace(/^https:\/\//i, 'wss://')
    .replace(/^http:\/\//i, 'ws://')
    .replace(/^\/+/, `${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/`);
}

export interface WsStreamOptions {
  /** 端点路径（如 /api/game/record） */
  path: string;
  /** 查询参数（undefined / null 项丢弃） */
  query?: QueryParams;
  /** 服务端文本帧（控制回执）；data 为原文，JSON 解析归调用方 */
  onMessage?: (data: string) => void;
  /** 连接建立（可以开始 send） */
  onOpen?: () => void;
  /** 连接关闭（主动 close / 服务端关 / 异常断开都会到这，恰好一次） */
  onClose?: (info: { code: number; reason: string }) => void;
  /** 基础地址（缺省同源，走代理） */
  baseURL?: string;
}

export interface WsStreamHandle {
  /** 发帧：二进制为数据（推流），string 为控制消息；连接已关闭时静默丢弃 */
  send: (chunk: Blob | ArrayBuffer | string) => void;
  /** 主动关闭（code 1000），触发 onClose */
  close: () => void;
  /** 连接是否仍打开（可 send） */
  readonly open: boolean;
}

/** 打开一条推流连接；返回句柄。连接失败走 onClose（code 1006） */
export function openWsStream(options: WsStreamOptions): WsStreamHandle {
  const { path, query, onMessage, onOpen, onClose, baseURL } = options;
  let closed = false;
  const socket = new WebSocket(toWsUrl(baseURL, path, query));
  socket.binaryType = 'arraybuffer';

  socket.addEventListener('open', () => onOpen?.());
  socket.addEventListener('message', (event) => {
    if (typeof event.data === 'string') onMessage?.(event.data);
    // 二进制下行不在本客户端语义内（推流是上行），忽略
  });
  socket.addEventListener('close', (event) => {
    if (closed) return; // 主动 close 已回调过
    closed = true;
    onClose?.({ code: event.code, reason: event.reason });
  });

  return {
    send(chunk) {
      if (socket.readyState !== WebSocket.OPEN) return;
      socket.send(chunk);
    },
    close() {
      if (closed) return;
      closed = true;
      socket.close(1000, 'client-stop');
      onClose?.({ code: 1000, reason: 'client-stop' });
    },
    get open() {
      return socket.readyState === WebSocket.OPEN;
    },
  };
}
