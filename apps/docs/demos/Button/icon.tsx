import { Button } from '@agent-packages/shared-ui';
import { Stack } from '@agent-packages/shared-ui';

const PlusIcon = () => (
  <svg
    viewBox="0 0 24 24"
    width="16"
    height="16"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.2"
    strokeLinecap="round"
  >
    <path d="M12 5v14M5 12h14" />
  </svg>
);

const ArrowIcon = () => (
  <svg
    viewBox="0 0 24 24"
    width="16"
    height="16"
    fill="none"
    stroke="currentColor"
    strokeWidth="2.2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M5 12h14M13 6l6 6-6 6" />
  </svg>
);

export default function ButtonIcon() {
  return (
    <Stack direction="row" spacing={3} wrap>
      <Button>
        <PlusIcon />
        新建文档
      </Button>
      <Button variant="tinted">
        下一页
        <ArrowIcon />
      </Button>
      <Button variant="gray" aria-label="添加">
        <PlusIcon />
      </Button>
    </Stack>
  );
}
