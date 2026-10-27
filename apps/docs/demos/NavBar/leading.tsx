import { NavBar } from '@agent-packages/shared-ui';

function BackIcon() {
  return (
    <svg
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M15 18l-6-6 6-6" />
    </svg>
  );
}

export default function NavBarLeading() {
  return (
    <div className="rounded-xl border border-separator">
      <NavBar
        sticky={false}
        leading={
          <button
            type="button"
            aria-label="返回"
            className="flex h-9 w-9 items-center justify-center rounded-full text-tint transition active:scale-95"
          >
            <BackIcon />
          </button>
        }
        title="详情页"
      />
      <p className="p-4 text-sm text-secondary-label">页面内容区域</p>
    </div>
  );
}
