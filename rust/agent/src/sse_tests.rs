//! sse.rs 的单元测试（由内联 mod tests 迁入）。

use super::*;

/// 依次取完所有帧
fn drain(decoder: &mut SseDecoder) -> Vec<String> {
    std::iter::from_fn(|| decoder.next_frame()).collect()
}

#[test]
fn two_frames_in_one_push() {
    let mut decoder = SseDecoder::new();
    decoder.push(b"data: {\"a\":1}\n\ndata: {\"b\":2}\n\n");
    assert_eq!(drain(&mut decoder), [r#"{"a":1}"#, r#"{"b":2}"#]);
}

#[test]
fn frame_split_across_chunks() {
    let mut decoder = SseDecoder::new();
    decoder.push(b"data: {\"lo");
    assert_eq!(decoder.next_frame(), None);
    decoder.push(b"ng\":true}\n");
    assert_eq!(decoder.next_frame(), None); // 帧还没结束（缺空行）
    decoder.push(b"\ndata: [DONE]\n\n");
    assert_eq!(decoder.next_frame(), Some(r#"{"long":true}"#.into()));
    assert_eq!(decoder.next_frame(), Some("[DONE]".into()));
}

#[test]
fn crlf_and_bare_cr_endings() {
    let mut decoder = SseDecoder::new();
    decoder.push(b"data: one\r\n\rdata: two\r\rdata: three\n\n");
    assert_eq!(drain(&mut decoder), ["one", "two", "three"]);
}

/// \r 与 \n 跨 chunk（\r 是上一 chunk 最后一字节，\n 在下一 chunk 开头）
#[test]
fn cr_split_across_chunks() {
    let mut decoder = SseDecoder::new();
    decoder.push(b"data: x\r");
    decoder.push(b"\n\r\ndata: y\r\n\n");
    assert_eq!(drain(&mut decoder), ["x", "y"]);
}

#[test]
fn multiline_data_joined_with_newline() {
    let mut decoder = SseDecoder::new();
    decoder.push(b"data: line1\ndata:line2\ndata:  line3\n\n");
    assert_eq!(decoder.next_frame(), Some("line1\nline2\n line3".into()));
}

#[test]
fn comments_and_event_lines_ignored() {
    let mut decoder = SseDecoder::new();
    decoder.push(b": ping\nevent: content_block_delta\ndata: {\"type\":1}\n\n");
    assert_eq!(decoder.next_frame(), Some(r#"{"type":1}"#.into()));
}

/// 只有 event 没有 data 的帧不算帧，不能吃掉后面的空行语义
#[test]
fn frame_without_data_skipped() {
    let mut decoder = SseDecoder::new();
    decoder.push(b"event: ping\n\ndata: real\n\n");
    assert_eq!(drain(&mut decoder), ["real"]);
}

#[test]
fn colon_without_space_kept_in_value() {
    let mut decoder = SseDecoder::new();
    decoder.push(b"data: {\"url\":\"https://x\"}\n\n");
    assert_eq!(decoder.next_frame(), Some(r#"{"url":"https://x"}"#.into()));
}

/// EOF 后缓冲里的残帧（无终止空行）取不出来，也不影响已产出内容
#[test]
fn incomplete_trailing_frame_dropped() {
    let mut decoder = SseDecoder::new();
    decoder.push(b"data: full\n\ndata: half");
    assert_eq!(drain(&mut decoder), ["full"]);
}
