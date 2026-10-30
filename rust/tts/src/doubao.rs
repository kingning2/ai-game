//! 豆包官方语音（逆向网页版 TTS 协议）。
//!
//! 协议还原自社区逆向项目 `callmerio/doubao-tts`（原 Python 实现）：
//!
//! 1. WebSocket 连接 `wss://ws-samantha.doubao.com/samantha/audio/tts?...`，
//!    查询串携带音色 / 格式 / 语速 / 音调与设备指纹（随机 device_id / web_id）；
//! 2. 连接后先后发送 `{"event":"text","text":...}` 与 `{"event":"finish"}`；
//! 3. 服务端推二进制帧（音频块）与 JSON 控制帧（open_success / sentence_start /
//!    sentence_end / error）；30s 收不到任何消息或连接关闭即视为合成结束。
//!
//! 鉴权：需要浏览器登录 doubao.com 后的 Cookie（原样传入请求头）。
//! 这是非官方接口，随豆包网页版本更新可能失效；Cookie 属于用户自己的会话凭证。
//!
//! 与本地 kokoro 引擎的关系：本模块是独立云端引擎，不走模型资产装配，
//! 直接调用 [`synthesize`] 即可；音色是豆包官方声音。

use std::time::Duration;

use rand::Rng;
use serde_json::{Value, json};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::http::header::{
    ACCEPT_LANGUAGE, CACHE_CONTROL, COOKIE, ORIGIN, PRAGMA, USER_AGENT,
};

use crate::TtsError;

/// 豆包网页版 TTS WebSocket 端点
const WS_URL: &str = "wss://ws-samantha.doubao.com/samantha/audio/tts";
/// 单条消息接收超时（超时即认为合成结束，同 Python 实现）
const RECV_TIMEOUT: Duration = Duration::from_secs(30);

/// 常用音色：简称 → 完整音色 ID（还原 Python SPEAKERS 表）
pub const SPEAKERS: &[(&str, &str)] = &[
    // 女声
    ("taozi", "zh_female_taozi_conversation_v4_wvae_bigtts"),
    ("shuangkuai", "zh_female_shuangkuai_emo_v3_wvae_bigtts"),
    ("tianmei", "zh_female_tianmei_conversation_v4_wvae_bigtts"),
    ("qingche", "zh_female_qingche_moon_bigtts"),
    // 男声
    ("yangguang", "zh_male_yangguang_conversation_v4_wvae_bigtts"),
    ("chenwen", "zh_male_chenwen_moon_bigtts"),
    ("rap", "zh_male_rap_mars_bigtts"),
    // 多语言
    ("en_female", "en_female_sarah_conversation_bigtts"),
    ("en_male", "en_male_adam_conversation_bigtts"),
];

/// 默认音色（桃子 · 对话）
pub const DEFAULT_SPEAKER: &str = "zh_female_taozi_conversation_v4_wvae_bigtts";

/// 音色解析：完整 ID 原样返回；简称映射为完整 ID；空串回退默认
pub fn resolve_speaker(voice: &str) -> &'static str {
    let voice = voice.trim();
    if voice.is_empty() {
        return DEFAULT_SPEAKER;
    }
    for (short, full) in SPEAKERS {
        if *short == voice || *full == voice {
            return full;
        }
    }
    DEFAULT_SPEAKER
}

/// 豆包 TTS 配置
#[derive(Debug, Clone)]
pub struct DoubaoConfig {
    /// 浏览器登录 doubao.com 的 Cookie（必填，从 DevTools → Network → 任意请求 Headers 复制）
    pub cookie: String,
    /// 音色完整 ID（用 [`resolve_speaker`] 解析简称）
    pub speaker: String,
    /// 音频格式：`aac` 或 `mp3`
    pub format: String,
}

impl DoubaoConfig {
    /// 最小配置：仅 Cookie，其余全默认（桃子 / aac）
    pub fn new(cookie: impl Into<String>) -> Self {
        Self {
            cookie: cookie.into(),
            speaker: DEFAULT_SPEAKER.to_string(),
            format: "aac".to_string(),
        }
    }

    /// 音频 MIME（拼 Content-Type 用）
    pub fn mime(&self) -> String {
        format!("audio/{}", self.format)
    }
}

/// 合成结果：音频字节（aac/mp3 裸流，按配置格式拼接）
#[derive(Debug, Clone, Default)]
pub struct DoubaoResult {
    pub audio: Vec<u8>,
}

/// 豆包云端支持的音频格式（注意：不支持 mp3，需要 mp3 请拿 aac 后自行转码）
const SUPPORTED_FORMATS: &[&str] = &["aac", "wav", "pcm", "ogg_opus"];

/// 合成一段语音（阻塞式收集全部音频后返回；流式播放场景可按需改造为 channel 推送）
pub async fn synthesize(config: &DoubaoConfig, text: &str) -> Result<DoubaoResult, TtsError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(TtsError::InvalidInput("text 不能为空".to_string()));
    }
    if !SUPPORTED_FORMATS.contains(&config.format.as_str()) {
        return Err(TtsError::InvalidInput(format!(
            "豆包云端不支持格式 {}（可选 {}）；需要 mp3 请用 aac 合成后转码",
            config.format,
            SUPPORTED_FORMATS.join("/")
        )));
    }
    if config.cookie.trim().is_empty() {
        return Err(TtsError::InvalidInput(
            "缺少豆包 Cookie：设置环境变量 DOUBAO_COOKIE（浏览器登录 doubao.com 后从 DevTools 复制）"
                .to_string(),
        ));
    }

    let request = build_request(config)?;
    let (mut ws, _) = connect_async(request)
        .await
        .map_err(|err| TtsError::Ws(format!("连接失败：{err}")))?;

    // 发文本 + 结束信号（协议两步）
    let send = |event: Value| Message::text(event.to_string());
    use futures_util::SinkExt;
    ws.send(send(json!({ "event": "text", "text": text })))
        .await
        .map_err(ws_err("发送文本失败"))?;
    ws.send(send(json!({ "event": "finish" })))
        .await
        .map_err(ws_err("发送结束信号失败"))?;

    // 收流：二进制帧攒音频，JSON 帧处理控制事件
    let mut result = DoubaoResult::default();
    loop {
        let message = tokio::time::timeout(RECV_TIMEOUT, futures_util::StreamExt::next(&mut ws))
            .await
            .map_err(|_| TtsError::Ws("接收超时".to_string()))?;

        let Some(message) = message else {
            break; // 连接关闭 → 合成结束（同 Python ConnectionClosed 分支）
        };
        match message.map_err(ws_err("接收消息失败"))? {
            Message::Binary(chunk) => result.audio.extend_from_slice(&chunk),
            Message::Text(payload) => {
                let data: Value = serde_json::from_str(&payload)
                    .map_err(|err| TtsError::Ws(format!("JSON 帧解析失败：{err}")))?;
                handle_event(data)?;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    if result.audio.is_empty() {
        return Err(TtsError::Ws("未收到任何音频数据".to_string()));
    }
    Ok(result)
}

/// 单个 JSON 控制帧：事件分流（还原 Python 的事件分支与错误判定）。
/// 合成结束不靠显式事件，由连接关闭或 30s 收流超时触发（同 Python）。
fn handle_event(data: Value) -> Result<(), TtsError> {
    let event = data
        .get("event")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match event {
        "open_success" => {
            tracing::debug!("豆包 TTS 连接成功");
            Ok(())
        }
        // sentence_start/end：无实质信息（音频在二进制帧），留给默认分支
        "error" => Err(TtsError::Ws(format!(
            "服务端错误：{}",
            describe_doubao_error(
                data.get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            )
        ))),
        _ => {
            let code = data.get("code").and_then(Value::as_i64).unwrap_or(0);
            if code != 0 {
                Err(TtsError::Ws(format!(
                    "code={code}：{}",
                    describe_doubao_error(
                        data.get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                    )
                )))
            } else {
                Ok(())
            }
        }
    }
}

/// 构造带指纹与鉴权头的 WebSocket 请求（查询串顺序还原 Python 实现）
fn build_request(
    config: &DoubaoConfig,
) -> Result<tokio_tungstenite::tungstenite::http::Request<()>, TtsError> {
    let url = format!(
        "{WS_URL}?speaker={speaker}&format={format}&speech_rate=0&pitch=0\
         &version_code=20800&language=zh&device_platform=web&aid=497858&real_aid=497858\
         &pkg_type=release_version&device_id={device_id}&pc_version=2.46.3&web_id={web_id}\
         &tea_uuid={web_id}&region=&sys_region=&samantha_web=1&use-olympus-account=1\
         &web_tab_id={tab_id}",
        speaker = config.speaker,
        format = config.format,
        device_id = device_id(),
        web_id = device_id(),
        tab_id = uuid::Uuid::new_v4(),
    );

    let mut request = url
        .into_client_request()
        .map_err(|err| TtsError::Ws(format!("请求构造失败：{err}")))?;
    let headers = request.headers_mut();
    headers.insert(ORIGIN, HeaderValue::from_static("https://www.doubao.com"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(
        COOKIE,
        config
            .cookie
            .parse()
            .map_err(|err| TtsError::InvalidInput(format!("Cookie 含非法字符：{err}")))?,
    );
    Ok(request)
}

/// 服务端错误话术翻译：风控拦截（710022002 block）与临时拒绝（710020702）给可操作提示
fn describe_doubao_error(message: &str) -> &str {
    match message {
        "block" => {
            "豆包风控拦截：会话被临时封禁。请改用完整 Cookie（浏览器 F12 全量复制），或稍后重试"
        }
        "系统错误" => "豆包网关拒绝（多为风控冷却或参数风控）：稍等几秒重试",
        other => other,
    }
}

fn ws_err(context: &'static str) -> impl Fn(tokio_tungstenite::tungstenite::Error) -> TtsError {
    move |err| TtsError::Ws(format!("{context}：{err}"))
}

/// 设备指纹随机整数：[7.4e18, 7.5e18)（还原 Python randint 区间，非安全用途）
fn device_id() -> u64 {
    7_400_000_000_000_000_000 + rand::rng().random_range(0..100_000_000_000_000_000)
}

#[cfg(test)]
#[path = "doubao_tests.rs"]
mod tests;
