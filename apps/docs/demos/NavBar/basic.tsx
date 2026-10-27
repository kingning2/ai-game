import { NavBar } from '@agent-packages/shared-ui';

export default function NavBarBasic() {
  return (
    <div className="rounded-xl border border-separator">
      <NavBar sticky={false} title="设置" />
      <p className="p-4 text-sm text-secondary-label">页面内容区域</p>
    </div>
  );
}
