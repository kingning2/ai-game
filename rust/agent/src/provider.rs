use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::anthropic::AnthropicProvider;
use crate::message::{ChatRequest, ThinkingMode};
use crate::openai::OpenAiProvider;
use crate::stream::ChatStream;
use crate::{AgentError, AgentResult};

/// [`AgentProvider::chat_stream`] 的返回 future。
///
/// 手动装箱替代 `async_trait`（trait 保持 dyn 对象安全，且零额外依赖）：
/// 根因是 Rust 原生 async fn in trait 尚不 dyn 兼容
pub type ChatStreamFuture<'a> = Pin<Box<dyn Future<Output = AgentResult<ChatStream>> + Send + 'a>>;

/// 对接 agent 的统一抽象：协议可插拔。
///
/// 一个实现 = 一种对话协议（OpenAI 兼容 / Anthropic / 未来自定义），
/// 同协议不同厂商靠 `ProviderSettings` 的 base_url + api_key 区分。
/// 新增协议：新模块实现本 trait + 在 [`build_provider`] 加一个分发分支
pub trait AgentProvider: Send + Sync + std::fmt::Debug {
    /// 供应商标识（"openai" / "anthropic"），用于日志与状态展示
    fn id(&self) -> &'static str;

    /// 发起流式对话。HTTP 响应就绪即返回（非 2xx 在这一步就报错），
    /// 事件用 [`ChatStream::next_event`] 逐个拉取
    fn chat_stream<'a>(&'a self, request: ChatRequest) -> ChatStreamFuture<'a>;
}

/// 供应商选择（配置文件 `provider` 字段，小写序列化）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    #[default]
    OpenAi,
    Anthropic,
}

/// 单协议的连接参数（配置文件 / 前端设置 DTO 的 Rust 侧形态）
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProviderSettings {
    /// API 根地址（不含尾斜杠）。OpenAI 含 `/v1`，Anthropic 是站点根
    pub base_url: String,
    /// API Key；空串视为未配置（调用时报错，构建时放行）
    pub api_key: String,
    /// 默认模型（请求未显式指定时使用）
    pub model: String,
}

impl ProviderSettings {
    /// API Key 在调用时校验（sidecar 未配 Key 也要能启动）；文案策略在 [`AgentError::missing_key`]
    pub(crate) fn require_key(&self) -> AgentResult<()> {
        if self.api_key.is_empty() {
            Err(AgentError::missing_key())
        } else {
            Ok(())
        }
    }
}

/// agent 对接配置：由前端设置页维护，server 持久化到配置文件（JSON）。
///
/// 文件缺失 / 字段缺失 → 回退默认（容器级 `#[serde(default)]`：缺的字段逐个
/// 从 [`Default`] 取，两个协议的默认连接参数不会串）；加载与保存策略在
/// `state::AgentState`，本类型只负责形状与构建 provider
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AgentConfig {
    /// 当前生效的供应商
    pub provider: ProviderKind,
    /// OpenAI 兼容协议连接参数（豆包兼容模式 / DeepSeek / Kimi / OpenRouter 等）
    pub openai: ProviderSettings,
    /// Anthropic Messages 协议连接参数
    pub anthropic: ProviderSettings,
    /// 单次回复 token 上限（Anthropic 必填；OpenAI 透传 max_tokens）
    pub max_tokens: u32,
    /// 思考模式；None = 不发参数（跟随模型默认）。方舟系模型用统一
    /// `thinking: {type}` 参数表达（GLM 系部分模型不支持 disabled，由网关报错）
    pub thinking: Option<ThinkingMode>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: ProviderKind::default(),
            openai: ProviderSettings {
                base_url: "https://api.openai.com/v1".to_string(),
                api_key: String::new(),
                model: "gpt-4o-mini".to_string(),
            },
            anthropic: ProviderSettings {
                base_url: "https://api.anthropic.com".to_string(),
                api_key: String::new(),
                model: "claude-sonnet-4-5".to_string(),
            },
            max_tokens: 4096,
            thinking: None,
        }
    }
}

impl AgentConfig {
    /// 当前生效供应商的连接参数（透传给 provider 构造）
    fn active_settings(&self) -> &ProviderSettings {
        match self.provider {
            ProviderKind::OpenAi => &self.openai,
            ProviderKind::Anthropic => &self.anthropic,
        }
    }
}

/// 共享 HTTP 客户端（两家 provider 同参）。
/// `Err` 仅在 reqwest Client 构建失败时出现（TLS 后端初始化等环境问题）
pub(crate) fn http_client() -> AgentResult<reqwest::Client> {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        // 整体请求超时：SSE 流挂起（服务端无响应/网络抖动）时兜底，
        // 120s 足够思考型模型完成一次 512 token 内的决策
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| AgentError::Config(format!("HTTP 客户端构建失败：{e}")))
}

/// 按配置构建 provider（Arc<dyn AgentProvider>，AppState 里直接挂这个）。
/// `Err` 仅在 HTTP 客户端构建失败时出现；未配 Key 不报错，调用时才报
pub fn build_provider(config: &AgentConfig) -> AgentResult<Arc<dyn AgentProvider>> {
    let settings = config.active_settings();
    if let Err(err) = settings.require_key() {
        tracing::warn!(error = %err, "provider 未配置 Key，调用时会报错");
    }
    let provider: Arc<dyn AgentProvider> = match config.provider {
        ProviderKind::OpenAi => Arc::new(OpenAiProvider::new(settings.clone(), config.max_tokens)?),
        ProviderKind::Anthropic => {
            Arc::new(AnthropicProvider::new(settings.clone(), config.max_tokens)?)
        }
    };
    Ok(provider)
}

#[cfg(test)]
#[path = "provider_tests.rs"]
mod tests;
