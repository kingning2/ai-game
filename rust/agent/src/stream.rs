use std::collections::VecDeque;

use crate::event::AgentEvent;
use crate::sse::SseDecoder;
use crate::{AgentError, AgentResult};

/// 单帧 data 负载 → 统一事件的解码函数（每协议一个，见 openai / anthropic 模块）
pub(crate) type FrameDecoder = fn(&mut StreamState, &str) -> AgentResult<Vec<AgentEvent>>;

/// 跨帧解码状态：用量/结束原因分散在不同帧里，收齐才能合成事件
#[derive(Debug, Default)]
pub(crate) struct StreamState {
    /// anthropic message_start 带的输入用量
    pub input_tokens: Option<u64>,
    /// openai finish_reason / anthropic stop_reason 原始值
    pub stop_reason: Option<String>,
    /// 已发过 Done（部分兼容网关会重复发结束帧，去重）
    pub done: bool,
}

/// 一次流式对话的事件流：pull 式消费 `next_event().await`
///
/// 刻意不实现 `futures::Stream`（省掉 futures 依赖）——server 侧用循环 +
/// mpsc channel 转 axum Sse；测试里直接循环拉取断言。
pub struct ChatStream {
    response: reqwest::Response,
    sse: SseDecoder,
    state: StreamState,
    decode: FrameDecoder,
    /// 单帧可能解出多个事件（如 anthropic message_delta 一次带用量+结束原因），排队逐个吐
    pending: VecDeque<AgentEvent>,
    /// 上游 EOF（或已收到 Done）后不再拉 chunk
    finished: bool,
}

impl ChatStream {
    pub(crate) fn new(response: reqwest::Response, decode: FrameDecoder) -> Self {
        Self {
            response,
            sse: SseDecoder::new(),
            state: StreamState::default(),
            decode,
            pending: VecDeque::new(),
            finished: false,
        }
    }

    /// 下一个事件；`Ok(None)` = 正常结束。`Err` = 流中断 / 协议错误
    pub async fn next_event(&mut self) -> AgentResult<Option<AgentEvent>> {
        loop {
            if let Some(event) = self.pending.pop_front() {
                return Ok(Some(event));
            }
            if self.finished {
                return Ok(None);
            }
            if let Some(data) = self.sse.next_frame() {
                let events = (self.decode)(&mut self.state, &data)?;
                if events
                    .iter()
                    .any(|event| matches!(event, AgentEvent::Done { .. }))
                {
                    // 上游已宣告结束：不必再等 EOF
                    self.finished = true;
                }
                self.pending.extend(events);
                continue;
            }
            match self.response.chunk().await {
                Ok(Some(bytes)) => self.sse.push(&bytes),
                // EOF：缓冲里的残帧（无终止空行）按 SSE 规范丢弃
                Ok(None) => self.finished = true,
                Err(error) => return Err(AgentError::Stream(error.into())),
            }
        }
    }
}

/// 非 2xx 检测点：只做 IO（读 body），错误的构造与截断策略在 [`AgentError::upstream`]
pub(crate) async fn ensure_success(response: reqwest::Response) -> AgentResult<reqwest::Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let body = response.text().await.unwrap_or_default();
    Err(AgentError::upstream(status, &body))
}

#[cfg(test)]
mod tests {
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
}
