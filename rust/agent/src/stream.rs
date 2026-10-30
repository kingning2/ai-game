use std::collections::VecDeque;

use crate::event::AgentEvent;
use crate::message::ToolCall;
use crate::sse::SseDecoder;
use crate::{AgentError, AgentResult};

/// 单帧 data 负载 → 统一事件的解码函数（每协议一个，见 openai / anthropic 模块）
pub(crate) type FrameDecoder = fn(&mut StreamState, &str) -> AgentResult<Vec<AgentEvent>>;

/// 跨帧聚合中的一个工具调用分片（openai 按 `index` 补位，anthropic 按 block index）
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ToolCallDraft {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// 跨帧解码状态：用量/结束原因/工具调用分片分散在不同帧里，收齐才能合成事件
#[derive(Debug, Default)]
pub(crate) struct StreamState {
    /// anthropic message_start 带的输入用量
    pub input_tokens: Option<u64>,
    /// openai finish_reason / anthropic stop_reason 原始值
    pub stop_reason: Option<String>,
    /// 已发过 Done（部分兼容网关会重复发结束帧，去重）
    pub done: bool,
    /// 工具调用分片（增量 id/name，arguments 字符串拼接）
    pub tool_calls: Vec<ToolCallDraft>,
}

impl StreamState {
    /// 定位（必要时补位）index 对应的聚合槽，供各协议写入分片
    pub(crate) fn tool_slot(&mut self, index: usize) -> &mut ToolCallDraft {
        if self.tool_calls.len() <= index {
            self.tool_calls.resize(index + 1, ToolCallDraft::default());
        }
        &mut self.tool_calls[index]
    }

    /// 聚合完毕 → 完整 ToolCall 列表（结束帧时调用；非 tool_calls 结束时为空）
    pub(crate) fn take_tool_calls(&mut self) -> Vec<ToolCall> {
        // 分片序即调用序，draft 转正式调用不丢信息
        self.tool_calls
            .drain(..)
            .map(|draft| ToolCall {
                id: draft.id,
                name: draft.name,
                arguments: draft.arguments,
            })
            .collect()
    }
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
#[path = "stream_tests.rs"]
mod tests;
