/**
 * @agent-packages/animation —— 狼人杀直播过场动画包（GSAP 时间线，全程零文字提示）。
 *
 * 消费方要求：
 * - Tailwind 扫描：消费方样式入口加 `@source '<包路径>/src'`（本包只用工具类，不自带 CSS）
 * - 语义令牌：arena-* / red / yellow 等令牌由消费方主题提供（对齐 apps/web 的 @theme）
 * - react / react-dom 为 peer；gsap / @gsap/react / shared-ui 随包依赖
 */
export { CountdownOverlay, NightfallOverlay, NIGHTFALL_MS } from './Overlays';
export { OpeningCeremony } from './OpeningCeremony';
export type { CeremonyPlayer, CeremonyRoster, OpeningCeremonyProps } from './OpeningCeremony';
