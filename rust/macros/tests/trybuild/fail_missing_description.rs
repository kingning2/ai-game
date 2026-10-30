// trybuild fail：缺少必填属性 description
use macros::agent_tool;

#[agent_tool(name = "incomplete")]
struct Incomplete {
    query: String,
}

fn main() {
    let _ = Incomplete::definition();
}
