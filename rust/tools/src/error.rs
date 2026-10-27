/// 工具调用失败的四分类。
///
/// 「未知工具 / 重复注册」由注册表报；「参数不合法 / 执行失败」由工具实现报。
/// 领域错误（如狼人杀的非法动作）由实现方折叠进 `Execution`，
/// 本包不依赖任何领域包。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ToolError {
    /// 注册表里没有这个名字
    #[error("未知工具：{name}")]
    UnknownTool { name: String },
    /// 同名工具重复注册
    #[error("工具「{name}」重复注册")]
    DuplicateTool { name: String },
    /// 入参与 Schema 不符（缺字段 / 类型不对）
    #[error("工具 {tool} 参数不合法：{message}")]
    InvalidArguments { tool: String, message: String },
    /// 参数合法但动作本身被领域规则拒绝（如阶段不符 / 目标已死）
    #[error("工具 {tool} 执行失败：{message}")]
    Execution { tool: String, message: String },
}

impl ToolError {
    /// 缺必填字段
    pub fn missing_field(tool: &str, field: &str) -> Self {
        Self::InvalidArguments {
            tool: tool.to_string(),
            message: format!("缺少必填字段「{field}」"),
        }
    }

    /// 字段类型或取值不对
    pub fn invalid_field(tool: &str, field: &str, expected: &str) -> Self {
        Self::InvalidArguments {
            tool: tool.to_string(),
            message: format!("字段「{field}」应为{expected}"),
        }
    }

    /// 领域错误折叠：实现方把自己的错误 `to_string()` 塞进来
    pub fn execution(tool: &str, message: impl Into<String>) -> Self {
        Self::Execution {
            tool: tool.to_string(),
            message: message.into(),
        }
    }
}
