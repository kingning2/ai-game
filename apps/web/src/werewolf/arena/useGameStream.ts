/**
 * useGameStream —— server 对局事件流的 React 封装（SSE 订阅 + ingest + 断线重连）。
 *
 * 数据流：streamGameStatus（apis/werewolf，SSE）→ 全量快照帧 → WerewolfGame.ingest
 * （@agent-packages/game，按 seq 去重累积 / playhead 回放 / 倒计时）→ useSyncExternalStore 快照。
 * 页面只消费快照与 phase/failure，不接触传输细节。
 */
import { useEffect, useState, useSyncExternalStore } from 'react';
import { WerewolfGame } from '@agent-packages/game';
import { SERVER_ERROR_CODE, toServerFailure, type ServerFailure } from '../../apis/http';
import { streamGameStatus } from '../../apis/werewolf';
import type { GameLocator, GameStatusDto } from '../../apis/types';

/** 连接状态：connecting=首帧未到；live=流正常；reconnecting=异常断开退避中；closed=终局/不可得 */
export type StreamPhase = 'connecting' | 'live' | 'reconnecting' | 'closed';

/** 异常断线重连间隔（对局是回合制，LLM 决策秒级；固定 3s 足够，无需指数退避） */
const STREAM_RETRY_MS = 3000;

export function useGameStream(locator: GameLocator | undefined, enabled = true) {
  // 实例放 state：StrictMode 模拟卸载会 destroy 实例，重挂渲染期发现已销毁就换新实例
  // （useMemo 复用已销毁实例会永久卡在等待页；渲染期守卫式替换是 React 官方认可的 adjust-state 模式）
  const [game, setGame] = useState(() => new WerewolfGame<GameStatusDto>());
  if (game.isDestroyed) {
    setGame(new WerewolfGame<GameStatusDto>());
  }
  useEffect(() => () => game.destroy(), [game]);
  const snapshot = useSyncExternalStore(game.subscribe, game.getSnapshot);
  const [phase, setPhase] = useState<StreamPhase>('connecting');
  const [failure, setFailure] = useState<ServerFailure | null>(null);

  // 定位参数只取标量做依赖，避免调用方每次渲染新对象导致重连
  const room = locator?.room;
  const gameId = locator?.game;

  useEffect(() => {
    if (!enabled) return undefined;
    let stopped = false;
    let stop: (() => void) | undefined;
    let retryTimer: ReturnType<typeof setTimeout> | undefined;
    const connect = (): void => {
      stop = streamGameStatus(
        { room, game: gameId },
        {
          onFrame: (status) => {
            setFailure(null);
            setPhase('live');
            game.ingest({ events: status.events, status });
          },
          onEnd: (reason, raw) => {
            stop = undefined;
            if (stopped) return;
            if (reason === 'ended') {
              setPhase('closed'); // 终局帧已 ingest，无需重连
              return;
            }
            if (reason === 'gone') {
              setPhase('closed');
              setFailure({
                kind: 'business',
                code: SERVER_ERROR_CODE.notFound,
                message: '对局不存在或已失效',
              });
              return;
            }
            if (reason === 'aborted') return; // 本地取消（卸载/重连），静默
            setPhase('reconnecting');
            setFailure(toServerFailure(raw ?? new Error('事件流中断')));
            retryTimer = setTimeout(connect, STREAM_RETRY_MS);
          },
        },
      );
    };
    connect();
    return () => {
      stopped = true;
      if (retryTimer !== undefined) clearTimeout(retryTimer);
      stop?.();
    };
  }, [room, gameId, enabled, game]);

  return { game, snapshot, phase, failure };
}
