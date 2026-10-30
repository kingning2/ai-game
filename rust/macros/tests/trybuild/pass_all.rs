// trybuild pass：全类型覆盖 + 异步宿主函数场景（契约 struct 本身不含函数）
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
use macros::{agent_tool, game_event};

#[agent_tool(name = "all_types", description = "全类型覆盖")]
struct AllTypes {
    text: String,
    flag: bool,
    seat: u8,
    big: u64,
    score: f64,
    maybe: Option<u32>,
    tags: Vec<String>,
}

#[agent_tool(name = "no_params", description = "无参工具")]
struct NoParams {}

#[game_event]
enum Events {
    Night { day: u32 },
    End,
}

fn main() {
    let _ = AllTypes::definition();
    let _ = NoParams::definition();
    let _ = Events::Night { day: 1 }.day_of();
    let _ = Events::End.day_of();
}
