// 豆包 TWS 完整协议探针：握手 → text+finish → 收流，汇总音频/错误。
// 用法：node doubao-probe.mjs <sessionid> [device_id] [format] [speaker]
const SESSION = process.argv[2] ?? '';
const DEV_ID = process.argv[3] ?? '7440000000000000009';
const FORMAT = process.argv[4] ?? 'aac';
const SPEAKER = process.argv[5] ?? 'zh_female_taozi_conversation_v4_wvae_bigtts';

const url =
  `wss://ws-samantha.doubao.com/samantha/audio/tts?speaker=${SPEAKER}&format=${FORMAT}` +
  `&speech_rate=0&pitch=0&version_code=20800&language=zh&device_platform=web&aid=497858` +
  `&real_aid=497858&pkg_type=release_version&device_id=${DEV_ID}&pc_version=2.46.3` +
  `&web_id=${DEV_ID}&tea_uuid=${DEV_ID}&region=&sys_region=&samantha_web=1` +
  `&use-olympus-account=1&web_tab_id=probe-${DEV_ID.slice(-3)}`;

const ws = new WebSocket(url, {
  headers: { Origin: 'https://www.doubao.com', Cookie: `sessionid=${SESSION}` },
});

let audio = 0,
  events = [],
  errored = false;
const finish = (tag) => {
  console.log(tag, JSON.stringify({ audio, events: events.slice(0, 6) }));
  process.exit(0);
};
const timer = setTimeout(() => finish('TIMEOUT'), 25000);

ws.addEventListener('open', () => {
  console.log('HANDSHAKE ok');
  ws.send(JSON.stringify({ event: 'text', text: '天黑请闭眼' }));
  ws.send(JSON.stringify({ event: 'finish' }));
});
ws.addEventListener('message', (ev) => {
  if (typeof ev.data === 'string') {
    const d = JSON.parse(ev.data);
    events.push(d.event ?? d.code ?? '?');
    if (d.event === 'error') {
      errored = true;
      clearTimeout(timer);
      finish('SERVER_ERROR');
    }
    if (d.event === 'open_success') console.log('SESSION open_success');
    if (d.event === 'sentence_start')
      console.log('SENTENCE:', d.sentence_start_result?.readable_text ?? '');
  } else audio += ev.data.size ?? 0;
});
ws.addEventListener('close', () => {
  clearTimeout(timer);
  finish(errored ? 'CLOSED(after error)' : audio ? 'DONE' : 'CLOSED(empty)');
});
ws.addEventListener('error', (e) => {
  clearTimeout(timer);
  finish('WS_ERROR ' + (e.message ?? '?'));
});
