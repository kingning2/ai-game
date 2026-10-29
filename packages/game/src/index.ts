/**
 * @agent-packages/game —— 对局前端基类（纯逻辑层，无 React / 无网络请求）。
 *
 * 分层规则（rules/common/package-rules.md）：本包只依赖 @agent-packages/shared，
 * 不含后端契约——对接后端的具体业务（DTO、端点、轮询）归调用方（apps/web 等），
 * 拉到数据后 ingest() 喂入本包的 WerewolfGame 基类。
 *
 * - enums：游戏动作 / 事件 / 角色 / 阵营等枚举
 * - types：事件流与舞台投影的最小类型（GameEventDto / GameFeed / ProjectedStage）
 * - playback：回放演出纯函数（事件停留时长、舞台投影、主持人口播）
 * - game：WerewolfGame 对局基类（事件队列、playhead 回放、倒计时、快照订阅）
 */
export * from './enums';
export * from './types';
export * from './playback';
export * from './game';
