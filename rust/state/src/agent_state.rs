use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use agent::{AgentConfig, AgentError, AgentProvider, ChatRequest, build_provider};

use crate::error::StateError;

/// 配置目录名（Windows %APPDATA%/… / macOS ~/Library/Application Support/… / Linux ~/.config/…）
pub(crate) const APP_DIR: &str = "agent-packages";

/// JSON 原子写（本 crate 各配置文件共用）：序列化 → 写 `.tmp` → rename 覆盖
/// （Windows 上 rename 同样支持替换已存在目标）
pub(crate) fn write_json_atomic<T: serde::Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), StateError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            StateError::persist(format!("创建配置目录 {} 失败", parent.display()), err)
        })?;
    }

    let json = serde_json::to_vec_pretty(value)
        .map_err(|err| StateError::persist("配置序列化失败", err))?;

    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|err| StateError::persist("配置写入失败", err))?;
    std::fs::rename(&tmp, path).map_err(|err| StateError::persist("配置替换失败", err))?;
    Ok(())
}

/// agent 运行时：当前配置 + 由它构建的 provider，前端设置页的读写目标。
///
/// - 配置来源：前端设置页经 server 端点下发，持久化到配置文件（JSON），
///   **不再读环境变量**；启动时从文件加载，缺失/损坏回退默认
/// - 锁：配置更新频率极低（用户改设置），对话请求只读 provider 快照，
///   std RwLock 短临界区即可；中毒时 `into_inner` 兜底（本仓库零 panic，实际不会中毒）
/// - 提交顺序：构建新 provider → 原子写文件 → 提交内存，任一步失败内存不变
#[derive(Debug)]
pub struct AgentState {
    /// 配置文件路径；None = 内存态（定位不到配置目录），更新只生效本次运行
    path: Option<PathBuf>,
    config: RwLock<AgentConfig>,
    provider: RwLock<Arc<dyn AgentProvider>>,
}

impl AgentState {
    /// 启动加载。永不失败：文件缺失/损坏回退默认（warn），
    /// provider 构建失败降级为 [`UnconfiguredProvider`]（调用时报 Config 错）
    pub fn load(path: PathBuf) -> Self {
        let config = std::fs::read(&path)
            .ok()
            .map(
                |bytes| match serde_json::from_slice::<AgentConfig>(&bytes) {
                    Ok(config) => config,
                    Err(err) => {
                        tracing::warn!(
                            error = %err,
                            path = %path.display(),
                            "agent 配置文件解析失败，回退默认配置（保存时覆盖）"
                        );
                        AgentConfig::default()
                    }
                },
            )
            .unwrap_or_default();

        if path.exists() {
            tracing::info!(path = %path.display(), "agent 配置文件加载成功");
        } else {
            tracing::info!(path = %path.display(), "agent 配置文件不存在，使用默认配置");
        }

        Self::build(config, Some(path))
    }

    /// 内存态：定位不到配置目录时的兜底，更新只生效本次运行
    pub fn ephemeral() -> Self {
        tracing::warn!("无法定位配置目录，agent 配置仅本次运行有效");
        Self::build(AgentConfig::default(), None)
    }

    /// 默认配置文件路径：`config_dir()/agent-packages/agent-config.json`
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join(APP_DIR).join("agent-config.json"))
    }

    /// 当前配置快照（GET 端点用；Key 由 DTO 层脱敏后才出边界）
    pub fn config(&self) -> AgentConfig {
        self.read_config().clone()
    }

    /// 当前 provider 快照（对话请求用，clone Arc 零成本）
    pub fn provider(&self) -> Arc<dyn AgentProvider> {
        Arc::clone(&self.read_provider())
    }

    /// 更新配置：构建新 provider → 原子写文件 → 提交内存。
    /// 任一步失败返回 Err 且内存保持旧值（provider 构建失败不会写坏运行状态）
    pub fn update(&self, config: AgentConfig) -> Result<(), StateError> {
        let provider = Self::build_provider_or_fallback(&config);

        if let Some(path) = &self.path {
            write_atomic(path, &config)?;
        }

        *self.write_config() = config;
        *self.write_provider() = provider;
        Ok(())
    }

    /// 组装：provider 构建失败不阻塞启动/更新，降级为 unconfigured
    fn build(config: AgentConfig, path: Option<PathBuf>) -> Self {
        let provider = Self::build_provider_or_fallback(&config);
        Self {
            path,
            config: RwLock::new(config),
            provider: RwLock::new(provider),
        }
    }

    /// provider 构建失败（TLS 后端初始化等极端环境问题）→ unconfigured 兜底
    fn build_provider_or_fallback(config: &AgentConfig) -> Arc<dyn AgentProvider> {
        match build_provider(config) {
            Ok(provider) => provider,
            Err(err) => {
                tracing::error!(error = %err, "provider 构建失败，降级为 unconfigured（调用时报错）");
                Arc::new(UnconfiguredProvider)
            }
        }
    }

    fn read_config(&self) -> RwLockReadGuard<'_, AgentConfig> {
        self.config
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn read_provider(&self) -> RwLockReadGuard<'_, Arc<dyn AgentProvider>> {
        self.provider
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write_config(&self) -> RwLockWriteGuard<'_, AgentConfig> {
        self.config
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write_provider(&self) -> RwLockWriteGuard<'_, Arc<dyn AgentProvider>> {
        self.provider
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn write_atomic(path: &Path, config: &AgentConfig) -> Result<(), StateError> {
    write_json_atomic(path, config)
}

/// provider 构建失败时的兜底实现：服务照常启动，调用时给出明确错误
#[derive(Debug)]
struct UnconfiguredProvider;

impl AgentProvider for UnconfiguredProvider {
    fn id(&self) -> &'static str {
        "unconfigured"
    }

    fn chat_stream<'a>(&'a self, _request: ChatRequest) -> agent::ChatStreamFuture<'a> {
        Box::pin(async {
            Err::<agent::ChatStream, AgentError>(AgentError::Config(
                "HTTP 客户端初始化失败，请重启应用".to_string(),
            ))
        })
    }
}

/// 跨模块测试辅助（仅测试构建可用）：唯一临时文件路径
#[cfg(test)]
pub(crate) mod tests_support {
    use std::path::PathBuf;

    pub(crate) fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "agent-state-test-{name}-{}.json",
            std::process::id()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        tests_support::temp_path(name)
    }

    #[test]
    fn load_missing_file_falls_back_to_default() {
        let path = temp_path("missing");
        let _ = std::fs::remove_file(&path);
        let state = AgentState::load(path);
        assert_eq!(state.config(), AgentConfig::default());
    }

    #[test]
    fn load_valid_file_roundtrips() {
        let path = temp_path("valid");
        let mut config = AgentConfig::default();
        config.openai.api_key = "sk-test-1234".to_string();
        config.max_tokens = 1024;

        write_atomic(&path, &config).unwrap();
        let state = AgentState::load(path.clone());
        assert_eq!(state.config().openai.api_key, "sk-test-1234");
        assert_eq!(state.config().max_tokens, 1024);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_corrupt_file_falls_back_to_default() {
        let path = temp_path("corrupt");
        std::fs::write(&path, b"{ not json").unwrap();
        let state = AgentState::load(path.clone());
        assert_eq!(state.config(), AgentConfig::default());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn update_persists_and_swaps_provider() {
        let path = temp_path("update");
        let state = AgentState::load(path.clone());

        let mut config = AgentConfig::default();
        config.provider = agent::ProviderKind::Anthropic;
        config.anthropic.api_key = "sk-ant-xyz".to_string();

        state.update(config).unwrap();

        // 内存已生效：provider 切到 anthropic
        assert_eq!(state.provider().id(), "anthropic");
        // 文件已落盘：重新加载能读回
        let reloaded = AgentState::load(path.clone());
        assert_eq!(reloaded.config().anthropic.api_key, "sk-ant-xyz");
        assert_eq!(reloaded.config().provider, agent::ProviderKind::Anthropic);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn update_failure_keeps_old_state() {
        let state = AgentState::ephemeral();
        let old = state.config();
        // ephemeral 不落盘，update 只会成功——这里验证的是"成功后内存生效"
        let mut config = AgentConfig::default();
        config.max_tokens = 256;
        state.update(config).unwrap();
        assert_eq!(state.config().max_tokens, 256);
        assert_eq!(old.max_tokens, 4096);
    }

    #[test]
    fn default_path_uses_config_dir() {
        if let Some(path) = AgentState::default_path() {
            assert!(path.to_string_lossy().contains(APP_DIR));
            assert!(path.to_string_lossy().ends_with("agent-config.json"));
        }
    }
}
