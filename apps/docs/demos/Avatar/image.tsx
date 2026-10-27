import { Avatar, HStack } from '@agent-packages/shared-ui';

/** 内联 SVG 生成 data URI 头像（demo 不依赖外网图片） */
function avatarSrc(color: string, initial: string) {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" fill="${color}"/><text x="32" y="41" font-size="26" text-anchor="middle" fill="#fff" font-family="sans-serif">${initial}</text></svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

export default function AvatarImage() {
  return (
    <HStack gap={3}>
      <Avatar src={avatarSrc('#0A84FF', 'L')} alt="林小临" fallback="林" />
      <Avatar src={avatarSrc('#30D158', 'Z')} alt="周小雨" fallback="周" />
      <Avatar src={avatarSrc('#FF9F0A', 'W')} alt="王一舟" fallback="王" />
      <Avatar fallback="空" />
    </HStack>
  );
}
