import { NavBar } from '@agent-packages/shared-ui';

export default function NavBarStatic() {
  return (
    <div className="rounded-xl border border-separator">
      <NavBar sticky={false} title="静态导航栏" />
      <p className="p-4 text-sm text-secondary-label">
        关闭 sticky 后不吸附滚动，作为普通静态栏参与文档流，适合卡片、弹层等局部场景。
      </p>
    </div>
  );
}
