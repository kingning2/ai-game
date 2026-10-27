import { NavBar } from '@agent-packages/shared-ui';

export default function NavBarCentered() {
  return (
    <div className="rounded-xl border border-separator">
      <NavBar
        sticky={false}
        leading={<span className="w-14" />}
        title="标题居中"
        trailing={
          <button
            type="button"
            className="w-14 text-right text-sm font-medium text-tint active:opacity-60"
          >
            编辑
          </button>
        }
      />
      <p className="p-4 text-sm text-secondary-label">页面内容区域</p>
    </div>
  );
}
