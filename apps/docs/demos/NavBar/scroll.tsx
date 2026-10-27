import { NavBar, Stack } from '@agent-packages/shared-ui';

export default function NavBarScroll() {
  return (
    <div className="h-56 overflow-y-auto rounded-xl border border-separator">
      <NavBar title="向下滚动，看毛玻璃" />
      <Stack gap={2} className="p-4">
        {Array.from({ length: 10 }, (_, i) => (
          <div
            key={i}
            className="flex h-10 items-center rounded-lg bg-tint/10 px-3 text-sm text-secondary-label"
          >
            {`内容块 ${i + 1}`}
          </div>
        ))}
      </Stack>
    </div>
  );
}
