import { useEffect, useState } from 'react';
import { Avatar, Spinner, cn } from '@agent-packages/shared-ui';
import { AGENT_POOL } from './boards';
import type { MvpDto } from './liveClient';

/** 头像映射：选手池按名字取图，找不到回退默认（姓氏末字缩写） */
function avatarOf(name: string): string | undefined {
  return AGENT_POOL.find((agent) => agent.name === name)?.avatarSrc;
}

/** 投票超时上限（秒）：投票由 server 异步执行（12 票并发 3/批），
 * 正常在几十秒内决出；超时说明投票任务失败，停止轮询提示并保持安静。
 * 刷新由对局页既有的 status 轮询（1.5s，快于约定的 10s 下限）驱动，此处不另发请求。 */
const MVP_TIMEOUT_S = 90;

/**
 * 结算面板 MVP 区块：pending 显示投票中（Spinner + 90s 超时提示）；
 * done 显示本场 MVP 与全部选票明细（voter → candidate，reason 次要文本）。
 * mvp 为 null（手动终局 / stale 中断局）不渲染，由调用方判断。
 */
export function MvpPanel({ mvp }: { mvp: MvpDto | null | undefined }) {
  // 投票中秒数：纯增量计数器驱动超时判定（interval 回调非渲染期，无 purity 问题）
  const [pendingSeconds, setPendingSeconds] = useState<number | null>(null);
  const pending = mvp !== null && mvp !== undefined && mvp.phase !== 'done';
  useEffect(() => {
    if (!pending) return undefined;
    const timer = setInterval(() => setPendingSeconds((s) => (s === null ? 0 : s + 1)), 1000);
    return () => {
      clearInterval(timer);
      setPendingSeconds(null);
    };
  }, [pending]);

  if (mvp === null || mvp === undefined) return null;

  if (mvp.phase !== 'done') {
    const timedOut = (pendingSeconds ?? 0) >= MVP_TIMEOUT_S;
    return (
      <div
        className={cn(
          'flex items-center justify-center gap-2 rounded-2xl border border-arena-line bg-arena-raised px-6 py-3',
          'text-sm text-arena-muted',
        )}
      >
        {timedOut ? (
          <span>⏱ MVP 统计超时，本轮无 MVP</span>
        ) : (
          <>
            <Spinner size="sm" />
            <span>MVP 投票中…全体选手正在评议会上的表现</span>
          </>
        )}
      </div>
    );
  }

  const winner = mvp.winner;
  return (
    <div
      className={cn(
        'flex w-full flex-col items-center gap-3 rounded-2xl border border-arena-gold/30 bg-arena-gold/10 px-6 py-4',
      )}
    >
      <p className="text-xs font-bold tracking-widest text-arena-gold/80">本场 MVP</p>
      <div className="flex items-center gap-3">
        <Avatar
          src={winner !== null ? avatarOf(winner) : undefined}
          alt={winner ?? '?'}
          fallback="★"
          size="lg"
        />
        <div className="text-left">
          <p className="text-xl font-black text-arena-text">{winner ?? '?'}</p>
          <p className="text-xs text-arena-muted">
            {mvp.votes.filter((v) => v.candidate === winner).length} / {mvp.votes.length} 票
          </p>
        </div>
      </div>
      {mvp.votes.length > 0 && (
        <ul className="flex max-h-40 w-full max-w-md flex-col gap-1 overflow-auto text-left text-xs">
          {mvp.votes.map((vote) => (
            <li key={vote.voter} className="flex items-start gap-2">
              <span className="shrink-0 text-arena-muted">
                {vote.voter} <span className="text-arena-muted">→</span>{' '}
                <span className="font-semibold text-arena-gold/90">{vote.candidate}</span>
              </span>
              {vote.reason !== '' && (
                <span className="min-w-0 truncate text-arena-muted">{vote.reason}</span>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
