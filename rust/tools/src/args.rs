use serde_json::Value;

use crate::error::ToolError;

/// 类型化参数提取器：包住工具入参 JSON，逐字段取出并做类型检查。
///
/// 失败统一折叠为 [`ToolError::InvalidArguments`]（带工具名），
/// 实现方不再手写「取字段 + 判型 + 报错」三段式。
#[derive(Debug, Clone)]
pub struct Args {
    tool: String,
    value: Value,
}

impl Args {
    /// 解析入参：必须是 JSON 对象（LLM 出参偶发数组/字符串，提前拦下）
    pub fn parse(tool: &str, value: Value) -> Result<Self, ToolError> {
        if !value.is_object() {
            return Err(ToolError::invalid_field(tool, "(整体)", "一个 JSON 对象"));
        }
        Ok(Self {
            tool: tool.to_string(),
            value,
        })
    }

    /// 必填无符号整数（座位号、开关等）
    pub fn u8(&self, field: &str) -> Result<u8, ToolError> {
        match self.opt_u8(field)? {
            Some(v) => Ok(v),
            None => Err(ToolError::missing_field(&self.tool, field)),
        }
    }

    /// 可选无符号整数（缺字段 / `null` 都算 `None`）
    pub fn opt_u8(&self, field: &str) -> Result<Option<u8>, ToolError> {
        match self.value.get(field) {
            None | Some(Value::Null) => Ok(None),
            Some(v) => v.as_u64().and_then(|n| u8::try_from(n).ok()).map_or_else(
                || {
                    Err(ToolError::invalid_field(
                        &self.tool,
                        field,
                        "0..=255 的整数",
                    ))
                },
                |n| Ok(Some(n)),
            ),
        }
    }

    /// 必填非空字符串（发言文本等；缺字段 / 空串都拒）
    pub fn string(&self, field: &str) -> Result<String, ToolError> {
        match self.opt_string(field)? {
            None => Err(ToolError::missing_field(&self.tool, field)),
            Some(text) if text.is_empty() => {
                Err(ToolError::invalid_field(&self.tool, field, "非空字符串"))
            }
            Some(text) => Ok(text),
        }
    }

    /// 可选字符串（缺字段 / `null` 都算 `None`；空串算 `Some("")` 由领域判）
    pub fn opt_string(&self, field: &str) -> Result<Option<String>, ToolError> {
        match self.value.get(field) {
            None | Some(Value::Null) => Ok(None),
            Some(v) => v.as_str().map(|s| Some(s.to_string())).map_or_else(
                || Err(ToolError::invalid_field(&self.tool, field, "字符串")),
                Ok,
            ),
        }
    }

    /// 必填布尔
    pub fn boolean(&self, field: &str) -> Result<bool, ToolError> {
        self.opt_boolean(field)?
            .ok_or_else(|| ToolError::missing_field(&self.tool, field))
    }

    /// 可选布尔
    pub fn opt_boolean(&self, field: &str) -> Result<Option<bool>, ToolError> {
        match self.value.get(field) {
            None | Some(Value::Null) => Ok(None),
            Some(v) => v.as_bool().map(Some).map_or_else(
                || Err(ToolError::invalid_field(&self.tool, field, "布尔值")),
                Ok,
            ),
        }
    }
}
