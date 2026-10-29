import {
  BrowserRouter,
  Navigate,
  Route,
  Routes,
  useLocation,
  useNavigate,
  useParams,
} from 'react-router';
import type { BoardDef, DealtGame } from './werewolf/boards';
import { BOARDS } from './werewolf/boards';
import type { Player } from './werewolf/types';
import { FlowDemo } from './werewolf/FlowDemo';
import { LobbyPage } from './werewolf/LobbyPage';
import { RoomSetup } from './werewolf/RoomSetup';
import { WerewolfPage } from './werewolf/WerewolfPage';
import { GameHubPage } from './werewolf/GameHubPage';
import { RosterSettingsPage } from './settings/RosterSettingsPage';
import { SettingsPage } from './settings/SettingsPage';
import { SettingsModal } from './settings/SettingsModal';
import { useState } from 'react';

/** /werewolf/setup/:boardId — 创建向导（选人 → 规则 → 发牌） */
function SetupRoute() {
  const { boardId } = useParams();
  const navigate = useNavigate();
  const board = BOARDS.find((candidate) => candidate.id === boardId);
  if (!board) {
    return <Navigate to="/werewolf" replace />;
  }
  return (
    <RoomSetup
      board={board}
      onBack={() => {
        void navigate('/werewolf');
      }}
      onComplete={(dealt: DealtGame) => {
        void navigate('/werewolf/play', {
          state: { boardId: board.id, roomCode: dealt.roomCode, players: dealt.players },
        });
      }}
    />
  );
}

/** /werewolf/play — 对局 / 观战；发牌结果与房间号经 location.state 传递（刷新后回退全明星板） */
function isPlayState(value: unknown): value is {
  boardId?: string;
  roomCode?: string;
  players?: Player[];
} {
  return typeof value === 'object' && value !== null;
}

function PlayRoute() {
  const navigate = useNavigate();
  const location = useLocation();
  const state = isPlayState(location.state) ? location.state : {};
  const board: BoardDef =
    BOARDS.find((candidate) => candidate.id === state.boardId) ?? BOARDS[BOARDS.length - 1];
  return (
    <FlowDemo
      board={board}
      roomName={state.roomCode !== undefined ? `${board.name} · ${state.roomCode}` : board.name}
      players={state.players}
      spectate={state.players === undefined}
      roomCode={state.roomCode}
      onBack={() => {
        void navigate('/werewolf');
      }}
    />
  );
}

/** 路由表：游戏页全屏沉浸；设置入口走弹层（SettingsModal），/settings 保留为独立深链页。 */
function AppRoutes() {
  const navigate = useNavigate();
  const [settingsOpen, setSettingsOpen] = useState(false);
  return (
    <>
      <Routes>
        <Route
          index
          element={
            <GameHubPage
              onEnterWerewolf={() => {
                void navigate('/werewolf');
              }}
              onSettings={() => setSettingsOpen(true)}
            />
          }
        />
        <Route
          path="werewolf"
          element={
            <LobbyPage
              onBack={() => {
                void navigate('/');
              }}
              onEnter={(board) => {
                void navigate(`/werewolf/setup/${board.id}`);
              }}
              onSpectateRoom={(board, roomCode) => {
                void navigate('/werewolf/play', { state: { boardId: board.id, roomCode } });
              }}
              onSpectate={() => {
                void navigate('/werewolf/spectate');
              }}
              onSettings={() => setSettingsOpen(true)}
            />
          }
        />
        <Route
          path="settings"
          element={
            <SettingsPage
              onBack={() => {
                void navigate('/');
              }}
              onOpenRoster={() => {
                void navigate('/settings/roster');
              }}
            />
          }
        />
        <Route path="settings/roster" element={<RosterSettingsPage />} />
        <Route path="werewolf/setup/:boardId" element={<SetupRoute />} />
        <Route path="werewolf/play" element={<PlayRoute />} />
        <Route
          path="werewolf/spectate"
          element={
            <WerewolfPage
              onBack={() => {
                void navigate('/werewolf');
              }}
            />
          }
        />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
      {/* 设置弹层：主页/大厅 ⚙️ 触发，全局单例 */}
      <SettingsModal open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </>
  );
}

/** 应用入口：真路由（react-router），URL 即状态 */
export default function App() {
  return (
    <BrowserRouter>
      <AppRoutes />
    </BrowserRouter>
  );
}
