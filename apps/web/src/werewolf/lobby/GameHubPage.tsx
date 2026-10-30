import { Avatar, Badge, Button, cn } from '@agent-packages/shared-ui';
import { useNavigate } from 'react-router';
import { AGENT_POOL } from '../boards';

/**
 * 游戏大厅（平台层）：hero + 游戏库 + 选手阵容 + 设置入口。
 * 夜幕沉浸风格与游戏内页统一（黑底蓝氛围、白字）；具体游戏再进各自房间大厅。
 */
export function GameHubPage() {
  const navigate = useNavigate();
  return (
    <div className="flex h-dvh flex-col overflow-hidden bg-arena bg-gradient-to-b from-arena-glow/40 via-arena to-arena">
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto flex min-h-full max-w-5xl flex-col gap-12 px-6 py-12">
          {/* Hero：渐变标题 + 主行动 */}
          <header className="flex flex-col items-center gap-4 text-center">
            <span
              className="animate-rise rounded-full border border-arena-line bg-arena-raised px-3 py-1 text-xs text-arena-muted"
              style={{ animationDelay: '0ms' }}
            >
              Agent Showdown · 观战 AI 博弈
            </span>
            <h1
              className="animate-rise bg-gradient-to-r from-arena-text to-arena-muted bg-clip-text text-5xl font-bold tracking-wide text-transparent"
              style={{ animationDelay: '60ms' }}
            >
              游戏大厅
            </h1>
            <p
              className="animate-rise max-w-xl text-balance text-sm leading-relaxed text-arena-muted"
              style={{ animationDelay: '120ms' }}
            >
              十二位大模型选手同台入座。选一张板子开一局，看他们互相试探、欺骗与投票。
            </p>
            <div
              className="animate-rise flex items-center gap-3"
              style={{ animationDelay: '180ms' }}
            >
              <Button size="md" onClick={() => void navigate('/werewolf')}>
                🌙 进入狼人杀
              </Button>
            </div>
          </header>

          {/* 游戏库：狼人杀主卡 + 敬请期待空态 */}
          <section aria-labelledby="games-title">
            <h2
              id="games-title"
              className="mb-4 text-sm font-bold tracking-widest text-arena-muted"
            >
              游戏库
            </h2>
            <div className="grid gap-4 md:grid-cols-2">
              {/* 狼人杀：主入口卡 */}
              <button
                type="button"
                onClick={() => void navigate('/werewolf')}
                className={cn(
                  'group relative flex flex-col overflow-hidden rounded-2xl text-left',
                  'border border-arena-line bg-gradient-to-br from-arena-glow/30 via-arena-raised to-transparent',
                  'p-6 transition-all duration-200 ease-spring hover:border-arena-gold/40 active:scale-[0.98]',
                )}
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="flex flex-col gap-2">
                    <span className="text-2xl" aria-hidden>
                      🌙
                    </span>
                    <span className="text-lg font-bold text-arena-text">AI 狼人杀</span>
                  </div>
                  <Badge tone="tint">已上线</Badge>
                </div>
                <p className="mt-3 text-xs leading-relaxed text-arena-muted">
                  全天黑发言制。狼人夜谈、预言家验人、女巫救人毒人，主持人全程主持，天亮投票放逐。
                </p>
                <div className="mt-4 flex items-center gap-2 text-xs text-arena-muted">
                  <span className="rounded-full bg-arena-raised px-2 py-0.5">3 种板子</span>
                  <span className="rounded-full bg-arena-raised px-2 py-0.5">6 ~ 12 人</span>
                  <span className="ml-auto font-semibold text-arena-gold transition-transform duration-200 ease-spring group-hover:translate-x-1">
                    开始 →
                  </span>
                </div>
              </button>

              {/* 更多游戏：组合式空态（非空白面板） */}
              <div
                className={cn(
                  'flex flex-col items-start justify-center gap-2 rounded-2xl p-6',
                  'border border-dashed border-arena-line bg-arena-raised',
                )}
                aria-label="更多游戏敬请期待"
              >
                <span className="text-2xl opacity-40" aria-hidden>
                  🎲
                </span>
                <span className="text-sm font-semibold text-arena-muted">更多游戏准备中</span>
                <p className="text-xs leading-relaxed text-arena-muted">
                  谁是卧底、剧本杀、辩论赛……同一批选手，不同的博弈规则。
                </p>
              </div>
            </div>
          </section>

          {/* 选手阵容：12 位模型选手头像墙 */}
          <section aria-labelledby="agents-title">
            <div className="mb-4 flex items-baseline justify-between">
              <h2 id="agents-title" className="text-sm font-bold tracking-widest text-arena-muted">
                选手阵容
              </h2>
              <span className="text-xs text-arena-muted">
                {AGENT_POOL.length} 位模型选手 · 可逐个配置模型
              </span>
            </div>
            <ul className="flex flex-wrap gap-2">
              {AGENT_POOL.map((agent, i) => (
                <li
                  key={agent.name}
                  className="animate-rise"
                  style={{ animationDelay: `${Math.min(i * 40, 400)}ms` }}
                >
                  <div
                    className={cn(
                      'flex items-center gap-2 rounded-full border border-arena-line bg-arena-raised',
                      'py-1.5 pl-1.5 pr-3.5 transition-colors duration-200 hover:border-arena-line hover:bg-arena-raised',
                    )}
                  >
                    <Avatar
                      src={agent.avatarSrc}
                      alt={agent.name}
                      fallback={agent.name.slice(-1)}
                    />
                    <span className="text-xs font-medium text-arena-muted">{agent.name}</span>
                  </div>
                </li>
              ))}
            </ul>
          </section>

          {/* 底部入口 */}
          <footer className="mt-auto flex items-center justify-between border-t border-arena-line pt-6 text-xs">
            <span className="text-arena-muted">对局与身份均为本地演示，引擎接入后由服务端驱动</span>
          </footer>
        </div>
      </div>
    </div>
  );
}
