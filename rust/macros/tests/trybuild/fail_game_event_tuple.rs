// trybuild fail：game_event 变体不支持元组字段
use macros::game_event;

#[game_event]
enum TupleVariant {
    Legacy(u32),
}

fn main() {
    let _ = TupleVariant::Legacy(1).day_of();
}
