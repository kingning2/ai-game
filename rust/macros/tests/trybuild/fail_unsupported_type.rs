// trybuild fail：不支持的自定义结构体参数类型 → 编译错误
use macros::agent_tool;

struct Product {
    id: u32,
}

#[agent_tool(name = "search_product", description = "搜索商品")]
struct SearchProduct {
    query: String,
    filter: Product,
}

fn main() {
    let _ = SearchProduct::definition();
}
