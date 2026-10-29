//! 状态层错误：持久化失败 / 名单外引用。
//!
//! state 包不做 HTTP 语义判断——错误冒泡到 server 后由
//! `server::error` 统一映射为 [`AppError`](../server/error/struct.AppError.html) 再渲染。

/// 通用 boxed 错误，用于携带底层 IO / 序列化原因
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// 配置与名单状态层的错误
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    /// 持久化失败（建目录 / 序列化 / 写入 / 替换），上下文在文案里、底层原因在 source 链
    #[error("配置持久化失败：{context}")]
    Persist {
        /// 失败环节描述（含路径等上下文）
        context: String,
        /// 底层 IO / 序列化错误
        #[source]
        source: BoxError,
    },

    /// 参数非法（空选手名 / 删除内置选手等），server 映射为 400
    #[error("{0}")]
    Invalid(String),
}

impl StateError {
    /// 便捷构造 [`StateError::Persist`]（自动装箱任意错误类型）
    pub fn persist(context: impl Into<String>, err: impl Into<BoxError>) -> Self {
        Self::Persist {
            context: context.into(),
            source: err.into(),
        }
    }
}
