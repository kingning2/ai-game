import { VStack, WerewolfStage, cn, type WerewolfPlayer } from '@agent-packages/shared-ui';
import { useState } from 'react';
import { HostPanel } from './HostPanel';
import { MemoryPanel } from './MemoryPanel';
import { MOCK_GAME } from './mock';
import { SpeechPanel } from './SpeechPanel';
import type { Player, Role } from './types';
import { useAutoSpeak, useVoice, type VoicePhase } from './useVoice';
import type { TtsEngine } from './speech';

/** 角色语义 → 舞台标签：狼红、其余黄（与引擎 Camp 两阵营对齐；上帝视角身份公开） */
const ROLE_META: Record<Role, { label: string; tone: 'wolf' | 'good' }> = {
  werewolf: { label: '狼人', tone: 'wolf' },
  seer: { label: '预言家', tone: 'good' },
  witch: { label: '女巫', tone: 'good' },
  guard: { label: '守卫', tone: 'good' },
  hunter: { label: '猎人', tone: 'good' },
  villager: { label: '平民', tone: 'good' },
};

/** 领域 Player → 舞台座位数据（seat 沿用 id；头像用官网 logo，无图回退首字；speaking 高亮当前发言者） */
function toStagePlayer(p: Player): WerewolfPlayer {
  return {
    seat: p.id,
    name: p.name,
    avatarSrc: p.avatarSrc,
    roleLabel: ROLE_META[p.role].label,
    roleTone: ROLE_META[p.role].tone,
    alive: p.status === 'alive',
    speaking: p.speaking,
  };
}

/** 语音开关按钮文案（按引擎区分；准备中/失败给不同引导） */
function voiceLabel(phase: VoicePhase, engine: TtsEngine): string {
  if (phase === 'off') return '🔊 启用语音';
  if (phase === 'preparing') {
    return engine === 'doubao' ? '⏳ 连接豆包…' : '⏳ 语音模型下载中…';
  }
  if (phase === 'on') return engine === 'doubao' ? '🔊 豆包音色已开启' : '🔊 语音已开启';
  return engine === 'doubao'
    ? '⚠️ 豆包不可用（服务端需配 DOUBAO_COOKIE），点击重试'
    : '⚠️ 语音模型下载失败，点击重试';
}

/** 狼人杀观战页：整页即舞台（fullBleed 沉浸全屏），引擎接入前由 mock 快照驱动，页面零状态 */
export function WerewolfPage({ onBack }: { onBack?: () => void }) {
  const {
    players,
    caption,
    host,
    hostMessage,
    currentSpeaker,
    wolfChat,
    phase: stagePhase,
    phaseLabel,
  } = MOCK_GAME;
  const speaker = players.find((p) => p.id === currentSpeaker?.playerId);
  const wolfSpeaker = players.find((p) => p.id === wolfChat?.playerId);
  const { phase, engine, setEngine, toggle } = useVoice();
  const [memoryOpen, setMemoryOpen] = useState(false);
  // 语音开启后，主持人主持词与选手台词变化即朗读（当前引擎合成）
  useAutoSpeak(phase, engine, hostMessage, speaker?.name, currentSpeaker?.text);

  return (
    <div className="relative h-dvh overflow-hidden">
      {/* 演示控制：返回大厅 / 语音引擎 / 语音开关 */}
      <div className="absolute top-4 right-4 z-10 flex items-center gap-2">
        {onBack && (
          <button
            type="button"
            onClick={onBack}
            className="rounded-full bg-arena/50 px-3 py-1.5 text-xs font-medium text-arena-text shadow-card backdrop-blur-sm hover:bg-arena/70"
          >
            ← 返回大厅
          </button>
        )}
        <button
          type="button"
          onClick={() => setEngine(engine === 'doubao' ? 'local' : 'doubao')}
          className="rounded-full bg-arena/50 px-3 py-1.5 text-xs font-medium text-arena-text shadow-card backdrop-blur-sm hover:bg-arena/70"
        >
          {engine === 'doubao' ? '🫘 豆包音色' : '📦 本地音色'}
        </button>
        <button
          type="button"
          onClick={() => setMemoryOpen((open) => !open)}
          className={cn(
            'rounded-full bg-arena/50 px-3 py-1.5 text-xs font-medium text-arena-text shadow-card backdrop-blur-sm',
            'hover:bg-arena/70',
          )}
        >
          {memoryOpen ? '🧠 收起记忆' : '🧠 玩家记忆'}
        </button>
        <button
          type="button"
          onClick={toggle}
          className={cn(
            'rounded-full bg-arena/50 px-3 py-1.5 text-xs font-medium text-arena-text shadow-card backdrop-blur-sm',
            'hover:bg-arena/70',
          )}
        >
          {voiceLabel(phase, engine)}
        </button>
      </div>
      <WerewolfStage
        title="AI 狼人杀"
        players={players.map(toStagePlayer)}
        phase={stagePhase}
        phaseLabel={phaseLabel}
        caption={caption}
        backgroundUrl="/werewolf/night-bg.jpg"
        fullBleed
      >
        <VStack gap={3} justify="center" className="h-full w-full">
          {host && hostMessage ? (
            <HostPanel host={host} message={hostMessage} className="w-full max-w-xl" />
          ) : null}
          {wolfSpeaker && wolfChat ? (
            <SpeechPanel
              speaker={wolfSpeaker}
              text={wolfChat.text}
              variant="wolf"
              className="w-full max-w-xl"
            />
          ) : speaker && currentSpeaker ? (
            <SpeechPanel speaker={speaker} text={currentSpeaker.text} className="w-full max-w-xl" />
          ) : null}
        </VStack>
      </WerewolfStage>
      {/* 玩家记忆浮层：某号眼里的对局事实 + LLM 压缩 */}
      {memoryOpen && (
        <MemoryPanel
          seats={players.length}
          onClose={() => setMemoryOpen(false)}
          className="absolute top-14 right-4 z-10 shadow-card"
        />
      )}
    </div>
  );
}
