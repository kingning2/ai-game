import { Button } from '@agent-packages/shared-ui';
import { useState } from 'react';
import { Outlet, useLocation, useNavigate } from 'react-router';
import { SettingsModal } from '../settings/SettingsModal';

/**
 * 应用壳（布局路由）：所有非全屏页面的共享 chrome。
 * - 返回导航：按路由层级推断目标，页面不再各自传 onBack
 * - ⚙️ 设置入口：常驻右上角，打开全局设置弹层（弹层状态收在壳内，App.tsx 不再管）
 * - 全屏沉浸页（对局 /werewolf/play）不包此壳，路由表里平级放置
 *
 * 返回目标表：路径前缀 → 返回去向；未命中则回主页。
 */
const BACK_TARGETS: Array<[prefix: string, to: string, label: string]> = [
  ['/werewolf', '/', '返回主页'],
];

export function AppLayout() {
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const back = BACK_TARGETS.find(([prefix]) => pathname.startsWith(prefix));

  return (
    <div className="relative h-dvh">
      {/* 共享 chrome：固定角落，不随页面滚动（页面内容区自带滚动） */}
      <div className="absolute inset-x-0 top-0 z-40 flex items-center justify-between px-4 py-3">
        {back ? (
          <Button variant="plain" size="sm" onClick={() => void navigate(back[1])}>
            ‹ {back[2]}
          </Button>
        ) : (
          <span />
        )}
        <Button variant="plain" size="sm" onClick={() => setSettingsOpen(true)}>
          ⚙️ AI 设置
        </Button>
      </div>
      <Outlet />
      {/* 设置弹层：壳级单例，壳内所有页面共用一个入口 */}
      <SettingsModal open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
}
