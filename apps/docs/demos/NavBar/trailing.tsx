import { NavBar } from '@agent-packages/shared-ui';

export default function NavBarTrailing() {
  return (
    <div className="rounded-xl border border-separator">
      <NavBar
        sticky={false}
        title="收件箱"
        trailing={
          <>
            <button type="button" className="text-sm font-medium text-tint active:opacity-60">
              全部已读
            </button>
            <button type="button" className="text-sm font-medium text-tint active:opacity-60">
              编辑
            </button>
          </>
        }
      />
      <p className="p-4 text-sm text-secondary-label">页面内容区域</p>
    </div>
  );
}
