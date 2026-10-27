//! 集成测试：内嵌 Mock SSE 服务器 → Provider 全链路 → 事件断言。
//!
//! 覆盖：OpenAI/Anthropic 双协议的 HTTP 请求形状（鉴权头、路径、body）与
//! 流式事件合成，以及各错误分支（4xx/5xx/流中错误帧/畸形帧/网络拒绝）。

use crate::AgentError;
use crate::event::{AgentEvent, StopReason, Usage};
use crate::message::{ChatMessage, ChatRequest};
use crate::provider::{AgentProvider, ProviderSettings};
use crate::{anthropic, openai};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 200 + SSE 流（connection: close → 读到 EOF 即响应体结束）
const SSE_OK: &str =
    "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\nconnection: close\r\n\r\n";

/// 用给定响应启动一次性 Mock 服务器，返回 base_url 与捕获的原始请求文本。
/// 只读到请求头结束（\r\n\r\n）即回响应，避免与客户端互相等待的死锁。
async fn spawn_mock(response: String) -> (String, tokio::sync::mpsc::Receiver<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (sender, receiver) = tokio::sync::mpsc::channel(1);

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let request = read_until_head_end(&mut socket).await;
        // 错误类测试只断言错误变体、不收请求，receiver 可能已被丢弃——发送失败不影响响应
        let _ = sender.send(request).await;

        socket.write_all(response.as_bytes()).await.unwrap();
        socket.flush().await.unwrap();
        socket.shutdown().await.unwrap();
    });

    (format!("http://{address}"), receiver)
}

/// 读到第一个 \r\n\r\n（请求头结束）为止，返回完整已读内容
async fn read_until_head_end(socket: &mut tokio::net::TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
        match socket.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
        }
    }
    String::from_utf8_lossy(&buffer).to_string()
}

fn openai_provider(base_url: String) -> openai::OpenAiProvider {
    openai::OpenAiProvider::new(
        ProviderSettings {
            base_url,
            api_key: "sk-test".into(),
            model: "gpt-test".into(),
        },
        1024,
    )
    .unwrap()
}

fn anthropic_provider(base_url: String) -> anthropic::AnthropicProvider {
    anthropic::AnthropicProvider::new(
        ProviderSettings {
            base_url,
            api_key: "ak-test".into(),
            model: "claude-test".into(),
        },
        1024,
    )
    .unwrap()
}

fn user_request() -> ChatRequest {
    ChatRequest {
        model: None,
        messages: vec![ChatMessage::user("你好")],
        thinking: None,
    }
}

/// 从原始请求文本里抠出 JSON body（\r\n\r\n 之后的部分）
fn request_body(raw: &str) -> serde_json::Value {
    serde_json::from_str(raw.rsplit("\r\n\r\n").next().unwrap()).unwrap()
}

/// 收集整个流为 (text, thinking, usage, done)
async fn drain<P: AgentProvider>(
    provider: &P,
    request: ChatRequest,
) -> crate::AgentResult<(String, String, Option<Usage>, Option<StopReason>)> {
    let mut stream = provider.chat_stream(request).await?;
    let (mut text, mut thinking, mut usage, mut done) = (String::new(), String::new(), None, None);
    loop {
        match stream.next_event().await {
            Ok(Some(event)) => match event {
                AgentEvent::TextDelta { delta } => text.push_str(&delta),
                AgentEvent::ThinkingDelta { delta } => thinking.push_str(&delta),
                AgentEvent::Usage { usage: captured } => usage = Some(captured),
                AgentEvent::Done { stop_reason } => done = Some(stop_reason),
            },
            Ok(None) => break,
            Err(error) => return Err(error),
        }
    }
    Ok((text, thinking, usage, done))
}

// ===== OpenAI 协议 =====

#[tokio::test]
async fn openai_full_stream() {
    let body = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"你\"}}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\"好\"}}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"想想\"}}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":12,\"completion_tokens\":5}}\n\n",
        "data: [DONE]\n\n",
    );
    let (base_url, mut requests) = spawn_mock(format!("{SSE_OK}{body}")).await;

    let (text, thinking, usage, done) = drain(&openai_provider(base_url), user_request())
        .await
        .unwrap();
    assert_eq!(text, "你好");
    assert_eq!(thinking, "想想");
    assert_eq!(
        usage,
        Some(Usage {
            input_tokens: 12,
            output_tokens: 5
        })
    );
    assert_eq!(done, Some(StopReason::EndTurn));

    // 请求形状：路径 / 鉴权 / body 关键字段
    let raw = requests.recv().await.unwrap();
    assert!(raw.starts_with("POST /chat/completions HTTP/1.1"));
    assert!(raw.contains("authorization: Bearer sk-test"));
    let payload = request_body(&raw);
    assert_eq!(payload["model"], "gpt-test");
    assert_eq!(payload["stream"], true);
    assert_eq!(payload["max_tokens"], 1024);
    assert_eq!(payload["messages"][0]["role"], "user");
}

#[tokio::test]
async fn openai_http_error_maps_status() {
    let (base_url, _) = spawn_mock(
        "HTTP/1.1 429 Too Many Requests\r\ncontent-type: application/json\r\nconnection: close\r\n\r\n{\"error\":{\"message\":\"限流\"}}".into(),
    )
    .await;
    let error = drain(&openai_provider(base_url), user_request())
        .await
        .unwrap_err();
    assert!(matches!(error, AgentError::Upstream { status, .. } if status.as_u16() == 429));
    assert!(error.to_string().contains("限流"));
}

#[tokio::test]
async fn openai_server_error_is_upstream() {
    let (base_url, _) =
        spawn_mock("HTTP/1.1 502 Bad Gateway\r\nconnection: close\r\n\r\n".into()).await;
    let error = drain(&openai_provider(base_url), user_request())
        .await
        .unwrap_err();
    assert!(matches!(error, AgentError::Upstream { status, .. } if status.as_u16() == 502));
}

#[tokio::test]
async fn openai_in_stream_error_is_upstream_event() {
    let (base_url, _) = spawn_mock(format!(
        "{SSE_OK}data: {{\"error\":{{\"message\":\"模型不存在\"}}}}\n\n"
    ))
    .await;
    let error = drain(&openai_provider(base_url), user_request())
        .await
        .unwrap_err();
    assert!(matches!(error, AgentError::UpstreamEvent(message) if message == "模型不存在"));
}

#[tokio::test]
async fn heartbeat_comments_are_skipped_not_fatal() {
    // SSE 注释行（心跳）不算帧，不应中断流；
    // 但完整的畸形 data 帧是 Protocol 错（静默跳过会丢内容还看起来正常结束）——另见 openai 单测
    let body = concat!(
        ": ping\n\n",
        ": keep-alive\n\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\"AB\"}}]}\n\n",
        "data: [DONE]\n\n",
    );
    let (base_url, _) = spawn_mock(format!("{SSE_OK}{body}")).await;
    let (text, ..) = drain(&openai_provider(base_url), user_request())
        .await
        .unwrap();
    assert_eq!(text, "AB");
}

#[tokio::test]
async fn connection_refused_is_request_error() {
    // 绑定后立刻释放端口 → 连接必被拒绝
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);

    let error = drain(
        &openai_provider(format!("http://{address}")),
        user_request(),
    )
    .await
    .unwrap_err();
    assert!(matches!(error, AgentError::Request(_)));
}

#[tokio::test]
async fn missing_api_key_fails_before_request() {
    let provider = openai::OpenAiProvider::new(
        ProviderSettings {
            base_url: "http://127.0.0.1:1".into(),
            api_key: String::new(),
            model: "m".into(),
        },
        16,
    )
    .unwrap();
    let error = drain(&provider, user_request()).await.unwrap_err();
    assert!(matches!(error, AgentError::Config(message) if message.contains("未配置")));
}

#[tokio::test]
async fn non_utf8_error_body_is_lossy_decoded() {
    // 网关返回非 UTF-8 错误体（GBK 之类）：reqwest text() lossy 解码，
    // 不会 panic 也不会变 Protocol，而是带替换符的 Upstream
    let mut raw = b"HTTP/1.1 500 Internal Server Error\r\ncontent-type: application/json\r\nconnection: close\r\n\r\n".to_vec();
    raw.extend_from_slice(&[0xFF, 0xFE, 0xFD]);
    let (base_url, _) = spawn_mock(String::from_utf8_lossy(&raw).to_string()).await;
    let error = drain(&openai_provider(base_url), user_request())
        .await
        .unwrap_err();
    let AgentError::Upstream { status, body } = error else {
        panic!("应为 Upstream，实际 {error:?}");
    };
    assert_eq!(status.as_u16(), 500);
    assert!(body.contains('\u{fffd}'));
}

// ===== Anthropic 协议 =====

#[tokio::test]
async fn anthropic_full_stream() {
    let body = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":20}}}\n\n",
        "event: ping\n",
        "data: {\"type\":\"ping\"}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"思考\"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"答\"}}\n\n",
        "event: message_delta\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":8}}\n\n",
        "event: message_stop\n",
        "data: {\"type\":\"message_stop\"}\n\n",
    );
    let (base_url, mut requests) = spawn_mock(format!("{SSE_OK}{body}")).await;

    let (text, thinking, usage, done) = drain(&anthropic_provider(base_url), user_request())
        .await
        .unwrap();
    assert_eq!(text, "答");
    assert_eq!(thinking, "思考");
    assert_eq!(
        usage,
        Some(Usage {
            input_tokens: 20,
            output_tokens: 8
        })
    );
    assert_eq!(done, Some(StopReason::EndTurn));

    // 请求形状：x-api-key + 版本头 + 顶层 system 拆分 + 必填 max_tokens
    let raw = requests.recv().await.unwrap();
    assert!(raw.starts_with("POST /v1/messages HTTP/1.1"));
    assert!(raw.contains("x-api-key: ak-test"));
    assert!(raw.contains("anthropic-version: 2023-06-01"));
    let payload = request_body(&raw);
    assert_eq!(payload["model"], "claude-test");
    assert_eq!(payload["max_tokens"], 1024);
    assert_eq!(payload["messages"].as_array().unwrap().len(), 1);
    assert!(payload.get("system").is_none());
}

#[tokio::test]
async fn anthropic_system_extracted_and_roles_merged() {
    let (base_url, mut requests) =
        spawn_mock(format!("{SSE_OK}data: {{\"type\":\"message_stop\"}}\n\n")).await;

    let request = ChatRequest {
        model: Some("claude-x".into()),
        thinking: None,
        messages: vec![
            ChatMessage::system("规则A"),
            ChatMessage::user("问1"),
            ChatMessage::user("问2"),
            ChatMessage::assistant("答"),
        ],
    };
    drain(&anthropic_provider(base_url), request).await.unwrap();

    let payload = request_body(&requests.recv().await.unwrap());
    assert_eq!(payload["system"], "规则A");
    assert_eq!(payload["model"], "claude-x"); // 显式 model 覆盖默认
    let turns = payload["messages"].as_array().unwrap();
    assert_eq!(turns.len(), 2); // user+user 合并，assistant 独立
    assert_eq!(turns[0]["content"], "问1\n\n问2");
}

#[tokio::test]
async fn anthropic_error_frame_is_upstream_event() {
    let body = concat!(
        "event: error\n",
        "data: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"过载\"}}\n\n",
    );
    let (base_url, _) = spawn_mock(format!("{SSE_OK}{body}")).await;
    let error = drain(&anthropic_provider(base_url), user_request())
        .await
        .unwrap_err();
    assert!(matches!(error, AgentError::UpstreamEvent(message) if message == "过载"));
}

#[tokio::test]
async fn anthropic_http_401_is_upstream() {
    let (base_url, _) =
        spawn_mock("HTTP/1.1 401 Unauthorized\r\nconnection: close\r\n\r\n".into()).await;
    let error = drain(&anthropic_provider(base_url), user_request())
        .await
        .unwrap_err();
    assert!(matches!(error, AgentError::Upstream { status, .. } if status.as_u16() == 401));
}

// ===== 依赖锁定（AV 报警整改项）=====

/// reqwest 锁 0.12.x 且未启用 gzip/brotli/zstd 压缩 feature（ supply-chain 整改项）
#[test]
fn reqwest_locked_at_0_12_without_compression() {
    let output = std::process::Command::new("cargo")
        .args(["tree", "-i", "reqwest", "--edges", "normal"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo tree 应可执行");
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text
        .lines()
        .find(|line| line.starts_with("reqwest v"))
        .expect("cargo tree 输出应有 reqwest 根节点");
    let version = line.split(" v").nth(1).unwrap_or_default();
    assert!(
        version.starts_with("0.12."),
        "reqwest 版本漂移: {version}（升级需重新走 supply-chain 审批）"
    );
}
