import { SectionHeader } from '@agent-packages/shared-ui';

export default function SectionHeaderBasic() {
  return (
    <div className="rounded-xl border border-separator p-4">
      <SectionHeader>通用设置 General</SectionHeader>
      <p className="mt-3 text-sm text-secondary-label">
        分组标题固定 iOS 规格：13px / semibold / 字距 +0.06em /
        次要文字色，英文自动大写（GENERAL）。
      </p>
    </div>
  );
}
