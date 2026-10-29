use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::error::StateError;
use agent::{AgentConfig, ProviderKind, ProviderSettings, ThinkingMode};

use crate::agent_state::APP_DIR;

/// 选手名单（与前端 `AGENT_POOL`（apps/web/src/werewolf/boards.ts）按名对齐）。
/// 覆盖配置以此名单的选手名为键；名单外名字的覆盖一律拒绝
pub const ROSTER_PLAYERS: [&str; 12] = [
    "Claude Opus 5.5",
    "Kimi K3.1 Pro",
    "Gemini 4 Argon",
    "GPT-6.1",
    "Grok 4.7",
    "MiMo V2.6",
    "Qwen3.8 Max",
    "DeepSeek V4.1",
    "GLM-5.3",
    "MiniMax M3.1",
    "豆包 Seed 2.1",
    "Mistral Large 4",
];

/// 单协议连接参数的字段级覆盖：字段缺省（None）= 继承全局配置对应字段
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SettingsPatch {
    /// API 根地址；None = 继承全局
    pub base_url: Option<String>,
    /// 模型；None = 继承全局
    pub model: Option<String>,
    /// API Key；None = 继承全局，`Some("")` = 明确清空
    pub api_key: Option<String>,
}

impl SettingsPatch {
    fn is_empty(&self) -> bool {
        self.base_url.is_none() && self.model.is_none() && self.api_key.is_none()
    }
}

/// 选手级模型覆盖；全字段缺省 = 完全跟随全局（落盘时自动移除该空覆盖）
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlayerOverride {
    /// 供应商切换；None = 继承全局
    pub provider: Option<ProviderKind>,
    /// OpenAI 兼容协议参数覆盖
    pub openai: Option<SettingsPatch>,
    /// Anthropic 协议参数覆盖
    pub anthropic: Option<SettingsPatch>,
    /// 单次回复 token 上限；None = 继承全局
    pub max_tokens: Option<u32>,
    /// 思考模式；None = 继承全局（全局默认不发参数）
    pub thinking: Option<ThinkingMode>,
    /// 自定义头像（emoji 或图片 URL）；None = 无自定义。
    /// 内置 12 人的头像由前端静态资源提供，此字段主要供自建选手用
    pub avatar: Option<String>,
}

impl PlayerOverride {
    fn is_empty(&self) -> bool {
        self.provider.is_none()
            && self.openai.as_ref().is_none_or(SettingsPatch::is_empty)
            && self.anthropic.as_ref().is_none_or(SettingsPatch::is_empty)
            && self.max_tokens.is_none()
            && self.thinking.is_none()
            && self.avatar.is_none()
    }
}

/// 选手级模型配置：按选手名存字段级覆盖，[`RosterState::resolve`] 合并全局配置出有效值。
///
/// - 覆盖频率极低（用户在选手配置页保存），std RwLock 短临界区即可，
///   中毒时 `into_inner` 兜底（本仓库零 panic，实际不会中毒）
/// - 持久化：`config_dir()/agent-packages/roster.json`，原子写（写 `.tmp` 后 rename）；
///   文件缺失/损坏回退"全员跟随全局"（warn），保存时覆盖
#[derive(Debug)]
pub struct RosterState {
    /// 配置文件路径；None = 内存态（定位不到配置目录），更新只生效本次运行
    path: Option<PathBuf>,
    overrides: RwLock<HashMap<String, PlayerOverride>>,
}

impl RosterState {
    /// 启动加载。永不失败：文件缺失/损坏回退空覆盖（warn）
    pub fn load(path: PathBuf) -> Self {
        let overrides = std::fs::read(&path)
            .ok()
            .map(
                |bytes| match serde_json::from_slice::<HashMap<String, PlayerOverride>>(&bytes) {
                    Ok(overrides) => overrides,
                    Err(err) => {
                        tracing::warn!(
                            error = %err,
                            path = %path.display(),
                            "选手配置文件解析失败，全员回退跟随全局（保存时覆盖）"
                        );
                        HashMap::new()
                    }
                },
            )
            .unwrap_or_default();

        if path.exists() {
            tracing::info!(path = %path.display(), "选手配置文件加载成功");
        } else {
            tracing::info!(path = %path.display(), "选手配置文件不存在，全员跟随全局");
        }

        Self {
            path: Some(path),
            overrides: RwLock::new(overrides),
        }
    }

    /// 内存态：定位不到配置目录时的兜底，更新只生效本次运行
    pub fn ephemeral() -> Self {
        tracing::warn!("无法定位配置目录，选手配置仅本次运行有效");
        Self {
            path: None,
            overrides: RwLock::new(HashMap::new()),
        }
    }

    /// 默认配置文件路径：`config_dir()/agent-packages/roster.json`
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join(APP_DIR).join("roster.json"))
    }

    /// 按默认路径加载，定位不到配置目录时内存态兜底（启动入口用）
    pub fn from_default_dir() -> Self {
        match Self::default_path() {
            Some(path) => Self::load(path),
            None => Self::ephemeral(),
        }
    }

    /// 某选手的覆盖配置；None = 未覆盖（跟随全局）
    pub fn override_of(&self, name: &str) -> Option<PlayerOverride> {
        self.read()
            .get(name)
            .cloned()
            .filter(|player_override| !player_override.is_empty())
    }

    /// 设置/清除选手覆盖。`None` 或全空覆盖 = 清除（跟随全局）。
    /// 名单外选手视为自建选手直接入库（空名拒绝）
    pub fn set_override(
        &self,
        name: &str,
        player_override: Option<PlayerOverride>,
    ) -> Result<(), StateError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(StateError::Invalid("选手名不能为空".to_string()));
        }

        let next = player_override.filter(|player_override| !player_override.is_empty());

        {
            let mut overrides = self.write();
            if next.is_some() {
                overrides.insert(name.to_string(), next.unwrap_or_default());
            } else {
                overrides.remove(name);
            }
        }
        self.persist()?;
        Ok(())
    }

    /// 全员名单：内置 12 人在前，自建选手按名排序殿后
    pub fn player_names(&self) -> Vec<String> {
        let read = self.read();
        let mut extras: Vec<String> = read
            .keys()
            .filter(|name| !ROSTER_PLAYERS.contains(&name.as_str()))
            .cloned()
            .collect();
        extras.sort();
        ROSTER_PLAYERS
            .iter()
            .map(ToString::to_string)
            .chain(extras)
            .collect()
    }

    /// 删除自建选手（含覆盖与头像）；内置选手不可删
    pub fn remove_player(&self, name: &str) -> Result<(), StateError> {
        if ROSTER_PLAYERS.contains(&name) {
            return Err(StateError::Invalid(format!("内置选手不可删除：{name}")));
        }
        self.write().remove(name);
        self.persist()?;
        Ok(())
    }

    /// 合并出某选手的有效配置：全局为底，覆盖逐字段替换。
    /// 编排层用它构建该选手的 provider（`agent::build_provider`）
    pub fn resolve(&self, name: &str, global: &AgentConfig) -> AgentConfig {
        let Some(player_override) = self.override_of(name) else {
            return global.clone();
        };

        let mut resolved = global.clone();
        if let Some(provider) = player_override.provider {
            resolved.provider = provider;
        }
        if let Some(patch) = &player_override.openai {
            Self::apply_patch(&mut resolved.openai, patch);
        }
        if let Some(patch) = &player_override.anthropic {
            Self::apply_patch(&mut resolved.anthropic, patch);
        }
        if let Some(max_tokens) = player_override.max_tokens {
            resolved.max_tokens = max_tokens;
        }
        if let Some(thinking) = player_override.thinking {
            resolved.thinking = Some(thinking);
        }
        resolved
    }

    fn apply_patch(target: &mut ProviderSettings, patch: &SettingsPatch) {
        if let Some(base_url) = &patch.base_url {
            target.base_url = base_url.clone();
        }
        if let Some(model) = &patch.model {
            target.model = model.clone();
        }
        if let Some(api_key) = &patch.api_key {
            target.api_key = api_key.clone();
        }
    }

    fn persist(&self) -> Result<(), StateError> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        crate::agent_state::write_json_atomic(path, &*self.read())
    }

    fn read(&self) -> std::sync::RwLockReadGuard<'_, HashMap<String, PlayerOverride>> {
        self.overrides
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write(&self) -> std::sync::RwLockWriteGuard<'_, HashMap<String, PlayerOverride>> {
        self.overrides
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_state::tests_support::temp_path;

    fn global() -> AgentConfig {
        AgentConfig {
            openai: ProviderSettings {
                base_url: "https://global.example.com/v1".into(),
                api_key: "sk-global".into(),
                model: "global-model".into(),
            },
            anthropic: ProviderSettings {
                base_url: "https://anthropic.example.com".into(),
                api_key: String::new(),
                model: "claude-global".into(),
            },
            ..AgentConfig::default()
        }
    }

    #[test]
    fn resolve_without_override_returns_global() {
        let roster = RosterState::ephemeral();
        let resolved = roster.resolve("豆包 Seed 2.1", &global());
        assert_eq!(resolved, global());
    }

    #[test]
    fn resolve_merges_field_level_patch() {
        let roster = RosterState::ephemeral();
        roster
            .set_override(
                "豆包 Seed 2.1",
                Some(PlayerOverride {
                    openai: Some(SettingsPatch {
                        model: Some("doubao-seed-1-6".into()),
                        ..SettingsPatch::default()
                    }),
                    max_tokens: Some(8192),
                    ..PlayerOverride::default()
                }),
            )
            .unwrap();

        let resolved = roster.resolve("豆包 Seed 2.1", &global());
        // 覆盖的字段生效
        assert_eq!(resolved.openai.model, "doubao-seed-1-6");
        assert_eq!(resolved.max_tokens, 8192);
        // 未覆盖的字段继承全局
        assert_eq!(resolved.openai.base_url, "https://global.example.com/v1");
        assert_eq!(resolved.openai.api_key, "sk-global");
        // 全局原配置不被污染
        assert_eq!(global().openai.model, "global-model");
    }

    #[test]
    fn resolve_applies_provider_switch() {
        let roster = RosterState::ephemeral();
        roster
            .set_override(
                "Kimi K3.1 Pro",
                Some(PlayerOverride {
                    provider: Some(ProviderKind::Anthropic),
                    ..PlayerOverride::default()
                }),
            )
            .unwrap();

        let resolved = roster.resolve("Kimi K3.1 Pro", &global());
        assert_eq!(resolved.provider, ProviderKind::Anthropic);
        assert_eq!(resolved.anthropic.model, "claude-global");
    }

    #[test]
    fn setting_empty_override_clears_entry() {
        let roster = RosterState::ephemeral();
        roster
            .set_override(
                "豆包 Seed 2.1",
                Some(PlayerOverride {
                    max_tokens: Some(1024),
                    ..PlayerOverride::default()
                }),
            )
            .unwrap();
        assert!(roster.override_of("豆包 Seed 2.1").is_some());

        roster.set_override("豆包 Seed 2.1", None).unwrap();
        assert!(roster.override_of("豆包 Seed 2.1").is_none());
    }

    #[test]
    fn custom_player_can_be_created_and_removed() {
        let roster = RosterState::ephemeral();
        roster
            .set_override(
                "神秘嘉宾",
                Some(PlayerOverride {
                    avatar: Some("🎭".into()),
                    openai: Some(SettingsPatch {
                        model: Some("doubao-seed-2-1-pro".into()),
                        ..SettingsPatch::default()
                    }),
                    ..PlayerOverride::default()
                }),
            )
            .unwrap();
        // 自建选手进入名单，且排在内置 12 人之后
        let names = roster.player_names();
        assert_eq!(names.len(), 13);
        assert_eq!(names.last().unwrap(), "神秘嘉宾");
        assert_eq!(
            roster.override_of("神秘嘉宾").unwrap().avatar.as_deref(),
            Some("🎭")
        );

        // 空名拒绝
        let err = roster
            .set_override("  ", Some(PlayerOverride::default()))
            .unwrap_err();
        assert!(matches!(err, StateError::Invalid(_)));

        // 内置选手不可删，自建可删
        assert!(roster.remove_player("豆包 Seed 2.1").is_err());
        roster.remove_player("神秘嘉宾").unwrap();
        assert!(roster.override_of("神秘嘉宾").is_none());
        assert_eq!(roster.player_names().len(), 12);
    }

    #[test]
    fn persists_and_reloads_roundtrip() {
        let path = temp_path("roster-roundtrip");
        {
            let roster = RosterState::load(path.clone());
            roster
                .set_override(
                    "MiniMax M3.1",
                    Some(PlayerOverride {
                        openai: Some(SettingsPatch {
                            base_url: Some("https://api.deepseek.com/v1".into()),
                            api_key: Some("sk-r1".into()),
                            model: Some("deepseek-reasoner".into()),
                        }),
                        ..PlayerOverride::default()
                    }),
                )
                .unwrap();
        }

        let reloaded = RosterState::load(path.clone());
        let player_override = reloaded.override_of("MiniMax M3.1").expect("覆盖应已落盘");
        assert_eq!(
            player_override.openai.expect("openai 补丁应存在").model,
            Some("deepseek-reasoner".into())
        );
        std::fs::remove_file(&path).ok();
    }
}
