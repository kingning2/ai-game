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

use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    /// 语速 -1.0 ~ 1.0（0 正常，协议层 ×100 发送）
    pub speech_rate: f32,
    /// 音调 -1.0 ~ 1.0（0 正常，协议层 ×100 发送）
    pub pitch: f32,
    /// 语言
    pub language: String,
}

impl DoubaoConfig {
    /// 最小配置：仅 Cookie，其余全默认（桃子 / aac / 常速）
    pub fn new(cookie: impl Into<String>) -> Self {
        Self {
            cookie: cookie.into(),
            speaker: DEFAULT_SPEAKER.to_string(),
            format: "aac".to_string(),
            speech_rate: 0.0,
            pitch: 0.0,
            language: "zh".to_string(),
        }
    }

    /// 音频 MIME（拼 Content-Type 用）
    pub fn mime(&self) -> String {
        format!("audio/{}", self.format)
    }
}

/// 合成结果
#[derive(Debug, Clone, Default)]
pub struct DoubaoResult {
    /// 音频字节（aac/mp3 裸流，按配置格式拼接）
    pub audio: Vec<u8>,
    /// 服务端逐句回传的可读文本（调试 / 字幕用）
    pub sentences: Vec<String>,
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
                handle_event(data, &mut result)?;
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
fn handle_event(data: Value, result: &mut DoubaoResult) -> Result<(), TtsError> {
    let event = data
        .get("event")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match event {
        "open_success" => {
            tracing::debug!("豆包 TTS 连接成功");
            Ok(())
        }
        "sentence_start" => {
            let readable = data
                .pointer("/sentence_start_result/readable_text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            tracing::debug!(readable = %readable, "豆包 TTS 句子开始");
            result.sentences.push(readable);
            Ok(())
        }
        "sentence_end" => Ok(()),
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
    let mut rng = Rng::from_clock();
    let url = format!(
        "{WS_URL}?speaker={speaker}&format={format}&speech_rate={rate}&pitch={pitch}\
         &version_code=20800&language={language}&device_platform=web&aid=497858&real_aid=497858\
         &pkg_type=release_version&device_id={device_id}&pc_version=2.46.3&web_id={web_id}\
         &tea_uuid={web_id}&region=&sys_region=&samantha_web=1&use-olympus-account=1\
         &web_tab_id={tab_id}",
        speaker = config.speaker,
        format = config.format,
        rate = scaled(config.speech_rate),
        pitch = scaled(config.pitch),
        language = config.language,
        device_id = rng.next_id(),
        web_id = rng.next_id(),
        tab_id = rng.uuid_v4(),
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

/// 语速/音调协议编码：×100 取整（0 保持 0）
fn scaled(value: f32) -> i32 {
    if value == 0.0 {
        0
    } else {
        (value * 100.0) as i32
    }
}

fn ws_err(context: &'static str) -> impl Fn(tokio_tungstenite::tungstenite::Error) -> TtsError {
    move |err| TtsError::Ws(format!("{context}：{err}"))
}

/// 极简伪随机数（splitmix64）：只为生成设备指纹，非安全用途。
/// 种子取系统时钟纳秒 + 进程号，进程内推进保证连续两次取值不同。
struct Rng(u64);

impl Rng {
    fn from_clock() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        Rng(nanos ^ ((std::process::id() as u64) << 32) ^ 0x9E37_79B9_7F4A_7C15)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// 设备 ID：[7.4e18, 7.5e18) 内的随机整数（还原 Python randint 区间）
    fn next_id(&mut self) -> u64 {
        7_400_000_000_000_000_000 + self.next() % 100_000_000_000_000_000
    }

    /// UUID v4 字符串（自产随机位，按 RFC 版本/变体位修正）
    fn uuid_v4(&mut self) -> String {
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&self.next().to_le_bytes());
        bytes[8..].copy_from_slice(&self.next().to_le_bytes());
        format_uuid(bytes)
    }
}

fn format_uuid(mut bytes: [u8; 16]) -> String {
    bytes[6] = (bytes[6] & 0x0F) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3F) | 0x80; // variant 10
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_in_python_range_and_distinct() {
        let mut rng = Rng::from_clock();
        let a = rng.next_id();
        let b = rng.next_id();
        for id in [a, b] {
            assert!(
                (7_400_000_000_000_000_000..7_500_000_000_000_000_000).contains(&id),
                "id {id} 超出 Python randint 区间"
            );
        }
        assert_ne!(a, b, "连续两次取值应不同");
    }

    #[test]
    fn uuid_shape() {
        let uuid = Rng::from_clock().uuid_v4();
        assert_eq!(uuid.len(), 36);
        assert_eq!(&uuid[14..15], "4", "v4 版本位");
        assert!(matches!(&uuid[19..20], "8" | "9" | "a" | "b"), "变体位");
        let mut rng = Rng::from_clock();
        assert_ne!(rng.uuid_v4(), rng.uuid_v4());
    }

    #[test]
    fn url_carries_protocol_params() {
        let config = DoubaoConfig::new("sessionid=abc");
        let request = build_request(&config).expect("构造");
        let url = request.uri().to_string();
        assert!(url.starts_with(WS_URL), "{url}");
        assert!(url.contains("speaker=zh_female_taozi_conversation_v4_wvae_bigtts"));
        assert!(url.contains("format=aac"));
        assert!(url.contains("aid=497858"));
        assert!(url.contains("real_aid=497858"));
        assert!(url.contains("device_platform=web"));
        assert!(url.contains("speech_rate=0"));
        assert!(url.contains("web_tab_id="));
        // 鉴权与来源头
        assert_eq!(
            request.headers().get(ORIGIN).unwrap(),
            "https://www.doubao.com"
        );
        assert_eq!(request.headers().get(COOKIE).unwrap(), "sessionid=abc");
    }

    #[test]
    fn rate_and_pitch_scaled() {
        assert_eq!(scaled(0.0), 0);
        assert_eq!(scaled(0.5), 50);
        assert_eq!(scaled(-0.15), -15);
    }

    #[test]
    fn speaker_resolution() {
        assert_eq!(resolve_speaker("rap"), "zh_male_rap_mars_bigtts");
        assert_eq!(
            resolve_speaker("zh_female_taozi_conversation_v4_wvae_bigtts"),
            "zh_female_taozi_conversation_v4_wvae_bigtts"
        );
        assert_eq!(resolve_speaker(""), DEFAULT_SPEAKER);
        assert_eq!(resolve_speaker("未知"), DEFAULT_SPEAKER);
    }

    #[tokio::test]
    async fn rejects_missing_cookie_before_connecting() {
        let config = DoubaoConfig::new("");
        let err = synthesize(&config, "你好").await.expect_err("应拒绝");
        assert!(matches!(err, TtsError::InvalidInput(_)), "{err:?}");
    }

    #[tokio::test]
    async fn rejects_empty_text_before_connecting() {
        let config = DoubaoConfig::new("sessionid=abc");
        let err = synthesize(&config, "   ").await.expect_err("应拒绝");
        assert!(matches!(err, TtsError::InvalidInput(_)), "{err:?}");
    }

    #[tokio::test]
    async fn rejects_mp3_before_connecting() {
        // 豆包云端不支持 mp3，客户端直接拦下（不发起连接撞风控）
        let mut config = DoubaoConfig::new("sessionid=abc");
        config.format = "mp3".to_string();
        let err = synthesize(&config, "你好").await.expect_err("应拒绝");
        let TtsError::InvalidInput(msg) = &err else {
            panic!("应返回参数错误，实际 {err:?}");
        };
        assert!(msg.contains("mp3"), "{msg}");
    }

    #[test]
    fn describes_risk_control_errors() {
        assert!(describe_doubao_error("block").contains("风控"));
        assert!(describe_doubao_error("系统错误").contains("重试"));
        assert_eq!(describe_doubao_error("其他"), "其他");
    }
}
