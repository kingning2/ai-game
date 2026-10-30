import os, io
old = set("""./agent/src/anthropic.rs ./agent/src/error.rs ./agent/src/event.rs ./agent/src/message.rs ./agent/src/openai.rs ./agent/src/provider.rs ./agent/src/sse.rs ./agent/src/stream.rs ./db/src/lib.rs ./db/src/store.rs ./games/werewolf/src/board.rs ./games/werewolf/src/role/mod.rs ./games/werewolf/src/script.rs ./memory/src/transcript.rs ./server/src/error/app_error.rs ./server/src/error/mod.rs ./server/src/lifecycle/mod.rs ./server/src/orchestr/autoplay.rs ./server/src/orchestr/memory.rs ./server/src/orchestr/mvp.rs ./server/src/routes/agent_config/mod.rs ./server/src/routes/record.rs ./server/src/routes/rooms/mod.rs ./server/src/routes/roster/mod.rs ./server/src/routes/tts.rs ./server/src/routes/werewolf/mod.rs ./state/src/agent_state.rs ./state/src/app_state.rs ./state/src/config.rs ./state/src/game_state.rs ./state/src/roster_state.rs ./state/src/stats.rs ./state/src/track.rs ./tts/src/doubao.rs ./tts/src/edge.rs ./tts/src/voices.rs ./utils/src/env.rs""".split())
now = set()
for root, dirs, names in os.walk('.'):
    dirs[:] = [d for d in dirs if d != 'target']
    for name in names:
        if name.endswith('.rs'):
            p = os.path.join(root, name).replace(os.sep, '/')
            src = io.open(p, encoding='utf-8').read()
            if '#[cfg(test)]' in src and 'mod tests {' in src:
                now.add(p)
print('消失的:', sorted(old - now))
print('新增的:', sorted(now - old))
