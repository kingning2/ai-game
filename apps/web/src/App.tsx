import { useState } from 'react';
import type { BoardDef, DealtGame } from './werewolf/boards';
import { FlowDemo } from './werewolf/FlowDemo';
import { LobbyPage, type Room } from './werewolf/LobbyPage';
import { RoomSetup } from './werewolf/RoomSetup';
import { WerewolfPage } from './werewolf/WerewolfPage';
import { GameHubPage } from './werewolf/GameHubPage';
import { RosterSettingsPage } from './settings/RosterSettingsPage';
import { SettingsPage } from './settings/SettingsPage';

/** 演示场：游戏大厅 → 狼人杀房间大厅 → 创建向导（选人/规则/发牌）→ 对局 / 静态观战 / AI 设置 / 选手模型 */
export default function App() {
  const [view, setView] = useState<
    'lobby' | 'werewolf' | 'setup' | 'room' | 'spectate' | 'settings' | 'roster'
  >('lobby');
  const [room, setRoom] = useState<{ info: Room; board: BoardDef } | null>(null);
  const [dealt, setDealt] = useState<DealtGame | null>(null);

  if (view === 'werewolf') {
    return (
      <LobbyPage
        onBack={() => setView('lobby')}
        onEnter={(info, board) => {
          setRoom({ info, board });
          setDealt(null);
          setView('setup');
        }}
        onSpectate={() => setView('spectate')}
        onSettings={() => setView('settings')}
      />
    );
  }
  if (view === 'setup' && room) {
    return (
      <RoomSetup
        board={room.board}
        roomName={room.info.name}
        onBack={() => setView('werewolf')}
        onComplete={(result) => {
          setDealt(result);
          setView('room');
        }}
      />
    );
  }
  if (view === 'room' && room) {
    return (
      <FlowDemo
        board={dealt?.board ?? room.board}
        roomName={room.info.name}
        players={dealt?.players}
        onBack={() => setView('werewolf')}
      />
    );
  }
  if (view === 'spectate') {
    return <WerewolfPage onBack={() => setView('lobby')} />;
  }
  if (view === 'settings') {
    return <SettingsPage onBack={() => setView('lobby')} onOpenRoster={() => setView('roster')} />;
  }
  if (view === 'roster') {
    return <RosterSettingsPage onBack={() => setView('settings')} />;
  }
  return (
    <GameHubPage
      onEnterWerewolf={() => setView('werewolf')}
      onSpectate={() => setView('spectate')}
      onSettings={() => setView('settings')}
    />
  );
}
