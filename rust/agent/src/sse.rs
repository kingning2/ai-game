/// SSE（text/event-stream）帧解析器：跨 chunk 缓冲字节，逐帧产出 `data` 负载。
///
/// 只实现两家协议实际用到的子集（WHATWG 规范）：
/// - 行以 `\n` / `\r\n` / `\r` 结尾；帧以空行结束
/// - 行格式 `field: value`，冒号后**一个**空格按规范剥离；无冒号整行是字段名
/// - `:` 开头是注释行，忽略
/// - `data` 可多行，合并时用 `\n` 连接；`event`/`id`/`retry` 字段忽略——
///   OpenAI 只有 data 行，Anthropic 的 data JSON 自带 `type`，比 event 行更可靠
///
/// 缓冲按字节累积（EOF 前的残帧无终止空行，按规范丢弃），
/// 行只在完整时才转 String，因此 UTF-8 多字节字符跨 chunk 不会被切坏。
#[derive(Debug, Default)]
pub struct SseDecoder {
    buffer: Vec<u8>,
    data_lines: Vec<String>,
}

impl SseDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加一段上游字节流
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// 取下一个完整帧的 data 负载（多行已合并；无 data 的帧被跳过）。没有完整帧返回 `None`
    pub fn next_frame(&mut self) -> Option<String> {
        loop {
            let line = self.take_line()?;
            if line.is_empty() {
                // 空行 = 帧结束：有 data 才算一帧，否则（连续空行/纯注释帧）继续扫
                if self.data_lines.is_empty() {
                    continue;
                }
                let data = self.data_lines.join("\n");
                self.data_lines.clear();
                return Some(data);
            }
            // 注释行
            if line.starts_with(':') {
                continue;
            }
            let (field, value) = match line.split_once(':') {
                Some((field, value)) => (field, value.strip_prefix(' ').unwrap_or(value)),
                None => (line.as_str(), ""),
            };
            if field == "data" {
                self.data_lines.push(value.to_string());
            }
        }
    }

    /// 从缓冲取一行（不含行尾符）；没有完整行返回 `None`（半行留在缓冲等下个 chunk）
    fn take_line(&mut self) -> Option<String> {
        let index = self.buffer.iter().position(|&b| b == b'\n' || b == b'\r')?;
        let mut end = index;
        let mut skip = 1;
        // \r\n 算一个行尾
        if self.buffer[index] == b'\r' && self.buffer.get(index + 1) == Some(&b'\n') {
            skip = 2;
        }
        end += skip;
        let line_bytes: Vec<u8> = self.buffer.drain(..end).collect();
        Some(String::from_utf8_lossy(&line_bytes[..line_bytes.len() - skip]).into_owned())
    }
}

#[cfg(test)]
#[path = "sse_tests.rs"]
mod tests;
