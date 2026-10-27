use serde::Serialize;

/// 统一响应包裹：`code === 0` 表示成功
///
/// 与前端 `packages/shared/src/request/types.ts` 的 `ApiResponse<T>` 同名同构，
/// 全仓库仅此一处定义，server 侧直接 `use crate::error::ApiResponse`。
#[derive(Serialize, utoipa::ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> axum::Json<Self> {
        axum::Json(Self {
            code: 0,
            message: "ok".to_string(),
            data,
        })
    }
}

impl ApiResponse<()> {
    /// 错误响应的包裹：data 固定为 null
    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: (),
        }
    }
}
