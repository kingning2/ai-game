// trybuild fail：未知属性与重复属性
use macros::agent_tool;

#[agent_tool(name = "dup", description = "x", unknown_key = "y")]
struct UnknownAttr {
    query: String,
}

fn main() {
    let _ = UnknownAttr::definition();
}
