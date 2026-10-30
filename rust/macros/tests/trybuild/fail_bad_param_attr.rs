// trybuild fail：#[param] 未知键
use macros::agent_tool;

#[agent_tool(name = "bad_param", description = "x")]
struct BadParam {
    #[param(desc = "旧写法")]
    query: String,
}

fn main() {
    let _ = BadParam::definition();
}
