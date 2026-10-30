// trybuild fail：#[game_event] 不能作用于 struct
use macros::game_event;

#[game_event]
struct NotAnEnum {
    day: u32,
}

fn main() {
    let _ = NotAnEnum { day: 1 };
}
