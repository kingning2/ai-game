//! 工作区过程宏：编译期代码生成，不含任何运行时业务逻辑。
//!
//! - [`agent_tool`]：工具契约 struct → `definition()`（JSON Schema）+ `parse()`（参数反序列化）
//! - [`game_event`]：游戏事件 enum → `day()` 元数据分发（从变体字段推导）

use proc_macro::TokenStream;

mod agent_tool;
mod game_event;

/// 工具契约宏（作用于 struct，生成 agent 引擎所需的注册元数据与解析入口）。
///
/// ```ignore
/// #[macros::agent_tool(name = "seer_check", description = "查验一名玩家")]
/// struct SeerCheck {
///     #[param(description = "目标座位号")]
///     target: u8,
/// }
/// ```
///
/// 生成（原 struct 保持不变，仅追加 derive 与两个关联函数）：
/// - `SeerCheck::definition() -> agent::ToolDefinition`
/// - `SeerCheck::parse(arguments: &str) -> Option<Self>`
///
/// 参数类型仅支持 `String`/`bool`/整数/浮点 及其 `Option`/`Vec` 包装，
/// 其他类型产生指向该类型的编译错误（不静默生成错误代码）。
#[proc_macro_attribute]
pub fn agent_tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    agent_tool::expand(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// 游戏事件宏（作用于 enum，生成 `day()` 元数据分发）。
///
/// ```ignore
/// #[macros::game_event]
/// pub enum GameEvent {
///     NightBegin { day: u32 },
///     GameOver,
/// }
/// ```
///
/// 生成 `impl GameEvent { pub fn day_of(&self) -> Option<u32> }`：
/// 含 `day: u32` 字段的变体返回 `Some(day)`，其余返回 `None`。
/// 新增变体无需维护任何 match——漏标 day 字段会自然归入 `None`，
/// 且 serde 序列化格式完全不受影响。
#[proc_macro_attribute]
pub fn game_event(attr: TokenStream, item: TokenStream) -> TokenStream {
    game_event::expand(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
