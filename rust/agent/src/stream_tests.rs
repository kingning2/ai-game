//! stream.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;
use crate::event::StopReason;

/// 测试用解码器：把 data 当文本 delta，"[DONE]" 发 Done
fn test_decode(state: &mut StreamState, data: &str) -> AgentResult<Vec<AgentEvent>> {
    if data == "[DONE]" {
        state.done = true;
        return Ok(vec![AgentEvent::Done {
            stop_reason: StopReason::EndTurn,
        }]);
    }
    Ok(vec![AgentEvent::TextDelta {
        delta: data.to_string(),
    }])
}

#[tokio::test]
async fn yields_events_and_stops_after_done() {
    // 内存 body 走通 decoder 管道；真 socket / 分 chunk / 中断场景在 mock e2e 测试里
    let body = concat!(
        "data: 你\n\ndata: 好\n\ndata: [DONE]\n\n",
        "data: 不该出现的帧\n\n",
    );
    let response = reqwest::Response::from(
        http::Response::builder()
            .status(200)
            .header("content-type", "text/event-stream")
            .body(body.to_string())
            .unwrap(),
    );
    let mut stream = ChatStream::new(response, test_decode);

    let mut texts = Vec::new();
    let mut done = false;
    while let Some(event) = stream.next_event().await.unwrap() {
        match event {
            AgentEvent::TextDelta { delta } => texts.push(delta),
            AgentEvent::Done { .. } => done = true,
            _ => panic!("不该出现的事件: {event:?}"),
        }
    }
    // 收到 Done 即停，后面的帧不再消费
    assert_eq!(texts, ["你", "好"]);
    assert!(done);
}
