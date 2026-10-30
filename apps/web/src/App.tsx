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
import { AppLayout } from './layout/AppLayout';
import { WarfieldPage } from './werewolf/arena/WarfieldPage';
import { LobbyPage } from './werewolf/lobby/LobbyPage';
import { RoomSetup } from './werewolf/lobby/RoomSetup';
import { GameHubPage } from './werewolf/lobby/GameHubPage';
import { RosterSettingsPage } from './settings/RosterSettingsPage';
import { SettingsPage } from './settings/SettingsPage';

/** /werewolf/setup/:boardId — 创建向导（选人 → 规则 → 发牌）；返回大厅由 AppLayout 壳提供 */
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
    <WarfieldPage
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

/** 路由表：游戏域壳内共享 AppLayout（返回/⚙️/设置弹层）；设置域自含 NavBar；对局页全屏沉浸。 */
function AppRoutes() {
  return (
    <Routes>
      {/* 壳路由：游戏域三页共享 chrome，页面只留特有动作 */}
      <Route element={<AppLayout />}>
        <Route index element={<GameHubPage />} />
        <Route path="werewolf" element={<LobbyPage />} />
        <Route path="werewolf/setup/:boardId" element={<SetupRoute />} />
      </Route>
      {/* 设置域：标准 App 工具条页，自含导航（URL 直达深链） */}
      <Route path="settings" element={<SettingsPage />} />
      <Route path="settings/roster" element={<RosterSettingsPage />} />
      {/* 对局页：全屏沉浸，不包壳；观战 = 无发牌状态进 PlayRoute（SSE 追平直播） */}
      <Route path="werewolf/play" element={<PlayRoute />} />
      <Route path="werewolf/spectate" element={<Navigate to="/werewolf/play" replace />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
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
