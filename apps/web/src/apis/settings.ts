/**
 * 设置域端点函数 —— 全局 agent 配置（/api/agent/config）与选手 roster（/api/roster）。
 * 一函数一端点，返回已解包的业务数据（http 自带 ApiResponse 信封解包）；DTO 契约见 ./types.ts。
 */
import { http } from './http';
import type { AgentConfigDto, AgentConfigUpdate, PlayerOverrideDto, RosterPlayer } from './types';

/** 读取全局 agent 配置（GET /api/agent/config） */
export const fetchAgentConfigDto = () => http.get<AgentConfigDto>('/agent/config');

/** 保存全局 agent 配置（PUT /api/agent/config）；Key 留空 = 不下发（服务端保留现值） */
export const putAgentConfig = (body: AgentConfigUpdate) =>
  http.put<AgentConfigDto>('/agent/config', body);

/** 拉取全员选手配置（12 人；有效配置已合并覆盖） */
export const fetchRoster = () => http.get<RosterPlayer[]>('/roster');

/** 保存单个选手的覆盖（字段缺省 = 继承全局；全空对象 = 清除覆盖回全局）。名单外选手自动创建 */
export const putPlayerOverride = (name: string, override: PlayerOverrideDto) =>
  http.put<RosterPlayer>(`/roster/${encodeURIComponent(name)}`, override);

/** 删除自建选手（内置选手 400），返回删除后的全员列表 */
export const deletePlayer = (name: string) =>
  http.delete<RosterPlayer[]>(`/roster/${encodeURIComponent(name)}`);
