import { Button } from '@agent-packages/shared-ui';
import { useNavigate } from 'react-router';
import { RosterManager } from './RosterManager';

/**
 * 选手模型配置页（独立深链入口）：页壳 + RosterManager 内容。
 * 主入口是设置 Modal 的「选手模型」分节；本页供 URL 直达。
 */
export function RosterSettingsPage() {
  const navigate = useNavigate();
  return (
    <div className="flex h-dvh flex-col overflow-hidden bg-background">
      <header className="flex h-14 shrink-0 items-center gap-2 border-b border-separator px-5">
        <Button
          variant="plain"
          size="sm"
          onClick={() => {
            void navigate('/settings');
          }}
        >
          ‹ 返回设置
        </Button>
        <span className="flex-1 text-center text-sm font-semibold text-label">选手模型</span>
        <span className="w-20" aria-hidden />
      </header>
      {/* 内容区铺满剩余宽度：滚动只发生在这里（app frame 铁律，见 rules/web/app-design.md §3） */}
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="w-full px-8 py-8">
          <RosterManager />
        </div>
      </div>
    </div>
  );
}
