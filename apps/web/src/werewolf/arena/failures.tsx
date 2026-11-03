/**
 * server 故障 → 用户文案（等待页/仪式页/对局内错误条共用）。
 * 自 WarfieldPage 抽出：页面本体超 lint 行数上限，文案分支自成一文件。
 */
import { VStack } from '@agent-packages/shared-ui';
import { SERVER_ERROR_CODE, type ServerFailure } from '../../apis/http';

/** 等待页标题：故障文案 > 重连提示 > 冷启动连接（故障最具体，优先显示） */
export function waitingTitle(failureTitle: string | null, reconnecting: boolean): string {
  if (failureTitle !== null) return failureTitle;
  if (reconnecting) return '事件流中断，正在重连……';
  return '正在连接 server，等待引擎发牌……';
}

/** 等待提示：`show` 为 false 时渲染 null —— 连接空窗期静默，避免等待文案一闪而过 */
export function WaitingNotice(props: {
  show: boolean;
  title: string;
  hint?: string;
  onBack?: () => void;
}) {
  const { show, title, hint, onBack } = props;
  if (!show) return null;
  return (
    <VStack gap={3} className="items-center text-center">
      <p className="text-lg font-bold text-arena-text">{title}</p>
      {hint !== undefined && <p className="max-w-md text-sm text-arena-muted">{hint}</p>}
      {onBack && (
        <button
          type="button"
          onClick={onBack}
          className="rounded-full bg-arena-raised px-4 py-2 text-sm font-medium text-arena-text hover:bg-arena-raised"
        >
          ← 返回大厅
        </button>
      )}
    </VStack>
  );
}

/**
 * server 故障 → 等待页/对局内错误条文案。按 apis/http 的 ServerFailure 逐分支给具体提示，
 * 码值与 rust/server/src/error/app_error.rs 的 AppError 一一对应，禁止透传原始错误字符串。
 */
export function failureCopy(failure: ServerFailure): { title: string; hint?: string } {
  switch (failure.kind) {
    case 'network':
      return {
        title: '无法连接 server',
        hint: '请先启动后端：仓库根目录执行 pnpm server（或 rust/ 下 cargo run -p server）。',
      };
    case 'http':
      // Vite 代理连不上后端时回 502（空响应体，statusText 才是 Bad Gateway）
      return failure.status === 502
        ? {
            title: 'server 没有应答',
            hint: '请求已到达代理但 server 无响应，通常是 server 未启动或正在重启编译。',
          }
        : { title: `server 响应异常（HTTP ${failure.status}）` };
    case 'abort':
      return { title: 'server 请求超时', hint: 'server 可能正忙，将自动重试。' };
    case 'parse':
      return {
        title: 'server 响应格式异常',
        hint: '返回内容不是约定的数据包裹，请确认 server 与前端版本一致。',
      };
    case 'business':
      return businessFailureCopy(failure.code, failure.message);
    case 'unknown':
      return { title: failure.message || '未知错误' };
  }
  // switch 已覆盖 ServerFailure 全部分支，此处仅为 lint consistent-return 兜底
  return { title: '未知错误' };
}

/** 业务码分支：只枚举对局页实际会发生的服务端错误，其余回落服务端原文 */
function businessFailureCopy(code: number, message: string): { title: string; hint?: string } {
  switch (code) {
    case SERVER_ERROR_CODE.notFound:
      return { title: '没有进行中的对局', hint: '对局尚未创建或已结束，请回到大厅重新开局。' };
    case SERVER_ERROR_CODE.conflict:
      return { title: '对局状态冲突', hint: message };
    case SERVER_ERROR_CODE.badRequest:
    case SERVER_ERROR_CODE.unprocessable:
      return { title: '开局请求参数有误', hint: message };
    case SERVER_ERROR_CODE.tooManyRequests:
      return { title: '请求过于频繁', hint: '请稍等片刻再操作。' };
    case SERVER_ERROR_CODE.timeout:
      return { title: 'server 处理超时', hint: '稍后将自动重试。' };
    case SERVER_ERROR_CODE.internal:
      return {
        title: 'server 内部错误',
        hint: '详情见 server 日志；若涉及模型调用，请检查选手模型与 API Key 配置。',
      };
    case SERVER_ERROR_CODE.serviceUnavailable:
      return {
        title: '模型服务暂不可用',
        hint: '上游模型无响应或返回错误，请检查网络与选手模型配置。',
      };
    default:
      return { title: message };
  }
}
