use std::sync::Arc;
use std::time::Instant;

use crate::agent_state::AgentState;
use crate::config::AppConfig;
use crate::game_state::GameState;
use crate::roster_state::RosterState;
use crate::stats::Stats;

/// axum 共享状态：`Router::with_state(state)` 注入，handler 用 `State(state): State<AppState>` 提取
///
/// 全局状态管理中心：
/// - `config`：静态配置（env 加载，启动后不变）
/// - `agent`：agent 运行时（配置文件 + 可热更新的 provider，前端设置页的读写目标）
/// - `stats`：运行时统计（track 中间件自动采集）
/// - 未来加数据库连接池 / HTTP 客户端 / 缓存 / 会话表等，都挂在这里
///
/// 约定：只放 `Clone + Send + Sync` 的东西（axum 要求 state 可克隆共享）；
/// 需要共享可变的内部状态用原子类型或 `Arc<Mutex/RwLock<...>>`，并把锁封在字段私有 + 方法访问后面。
#[derive(Debug, Clone)]
pub struct AppState {
    /// 静态配置
    pub config: AppConfig,
    /// agent 运行时（配置读写 + provider 热更新）
    pub agent: Arc<AgentState>,
    /// 选手级模型覆盖（12 名选手各自的 provider/模型/Key，None 字段继承 `agent` 全局配置）
    pub roster: Arc<RosterState>,
    /// 对局控制面（狼人杀开局/暂停/继续/结束，全局唯一当前对局）
    pub game: Arc<GameState>,
    /// 本地语音合成（kokoro 模型进程内推理；狼人杀观战朗读台词）
    pub tts: Arc<tts::TtsService>,
    /// 全局请求统计（自动采集）
    pub stats: Stats,
    /// 服务启动时刻（用于 /api/info 的运行时长）
    started_at: Instant,
}

impl AppState {
    pub fn new(config: AppConfig, agent: AgentState) -> Self {
        Self::new_with_roster(config, agent, RosterState::from_default_dir())
    }

    /// 指定 RosterState 构造（测试/嵌入式注入内存态；正常启动走默认配置目录）
    pub fn new_with_roster(config: AppConfig, agent: AgentState, roster: RosterState) -> Self {
        Self {
            config,
            agent: Arc::new(agent),
            roster: Arc::new(roster),
            game: Arc::new(GameState::new()),
            tts: Arc::new(tts::TtsService::with_default_dir()),
            stats: Stats::default(),
            started_at: Instant::now(),
        }
    }

    /// 服务已运行秒数
    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uptime_starts_at_zero() {
        let state = AppState::new(AppConfig::default(), AgentState::ephemeral());
        assert_eq!(state.uptime_secs(), 0);
        assert_eq!(state.config.port, 17870);
        assert_eq!(state.stats.total(), 0);
    }
}
