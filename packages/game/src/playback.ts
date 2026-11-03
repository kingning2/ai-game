/**
 * 回放演出层（纯函数，无副作用）：
 * - playbackMs：每类事件的舞台停留时长
 * - projectStage：事件前缀 → 舞台视图（昼夜相位 / 聚光灯 / 发言 / 主持词）
 * - hostSay：主持人事件口播文案
 */
import { GameEventKind } from './enums';
import type { GameEventDto, GamePlayer, ProjectedStage } from './types';

/** 按事件类型生成舞台停留时长（ms）：转场长一些，发言按文本长度 pacing（约 11 字/秒） */
export function playbackMs(event: GameEventDto): number {
  switch (event.kind) {
    case GameEventKind.Night:
      return 3400; // 天黑请闭眼：夜幕渐变 + 主持人念完仪式台词
    case GameEventKind.Dawn:
      return 3400; // 天亮请睁眼 + 死讯揭晓前奏
    case GameEventKind.WolfChat:
    case GameEventKind.Speech:
      // 兑底估算（约 7.7 字/秒）：正常路径 CaptionBar 播完即提前推进（speechDone），
      // 只有 TTS 失败降级定时器时才会等到这里，所以放宽余量避免截断语音
      return Math.min(2600 + event.text.length * 130, 12000);
    case GameEventKind.Died:
      return 3800; // 死讯要停留够久，让座位打叉动画与默哀播完
    case GameEventKind.NightAction:
      return 2200;
    case GameEventKind.WolfPass:
      return 1600;
    case GameEventKind.Duel:
    case GameEventKind.Vote:
    case GameEventKind.Idiot:
    case GameEventKind.Hunter:
      return 2400;
    case GameEventKind.Over:
      return 4200; // 胜负定格
    default:
      return 1600;
  }
}

/** 夜里发生的事件（用于推导 playhead 处的昼夜相位；started 后紧跟天黑） */
const NIGHT_KINDS: ReadonlySet<string> = new Set([
  GameEventKind.Started,
  GameEventKind.Night,
  GameEventKind.NightAction,
  GameEventKind.WolfChat,
  GameEventKind.WolfPass,
]);

/**
 * 事件前缀 → 舞台视图：只看已播到的事件，而非服务端最新状态。
 * 服务端 autoplay（400ms/步）远快于观看节奏，界面必须按 seq 顺序逐条演出，
 * 否则刚进房间就直接跳到“天亮死人了”。
 */
export function projectStage<TPlayer extends GamePlayer>(
  events: GameEventDto[],
  players: TPlayer[],
  roomName: string,
  /** 未播的下一条事件（调用方从全量流取）：用于主持词衔接（如狼刀后接「狼人请闭眼」） */
  next?: GameEventDto,
): ProjectedStage<TPlayer> {
  const last = events.at(-1);
  const dead = new Set(
    events
      .filter((e) => e.kind === GameEventKind.Died)
      .map((e) => e.seat)
      .filter((s) => s !== null),
  );
  // 昼夜相位由事件本身推导（status.phase 是服务端最新相位，会超前于播放进度）。
  // 服务端先播死讯再播天亮：同一天的天亮还没演到，死讯仍属夜晚揭晓，相位不能提前翻昼
  const night = last
    ? NIGHT_KINDS.has(last.kind) ||
      (last.kind === GameEventKind.Died &&
        !events.some((e) => e.kind === GameEventKind.Dawn && e.day === last.day))
    : true;
  // 发言：前缀里最近一条带文本的事件（含说话座位）。夜晚只在“最后一条事件就是
  // 这条发言”时展示；狼聊不在这里——它是狼人内部交流，走 wolfChat 段并排展示，不进主发言栏
  const talk = events
    .toReversed()
    .find((e) => e.kind === GameEventKind.Speech && e.seat !== null && !(night && e !== last));
  // 狼人内部交流段：前缀末尾最近一段连续 WolfChat（中间被夜行动/狼沉默打断则截断）。
  // 全段同时展示——现实中是两狼同时商议，不是逐个演讲；播放层不播这段的语音
  const wolfChat: { player: TPlayer; text: string }[] = [];
  if (night) {
    for (const e of events.toReversed()) {
      if (e.kind !== GameEventKind.WolfChat || e.seat === null) break;
      const player = players.find((p) => p.id === e.seat);
      if (player) wolfChat.unshift({ player, text: e.text });
    }
  }
  // 夜晚聚光：前缀里最近一条夜行动/狼聊/狼沉默的主角
  const spotlightSeat =
    events
      .toReversed()
      .find(
        (e) =>
          e.seat !== null &&
          (e.kind === GameEventKind.NightAction ||
            e.kind === GameEventKind.WolfChat ||
            e.kind === GameEventKind.WolfPass),
      )?.seat ?? null;
  // 刀口：前缀里最近一条狼刀夜行动的 target（女巫环节报死讯用；空刀/未找到为 undefined）
  const wolfTarget = events
    .toReversed()
    .map((e) =>
      e.kind === GameEventKind.NightAction ? /狼刀(\d+)号/.exec(e.text)?.[1] : undefined,
    )
    .find((t) => t !== undefined);
  const day = last?.day ?? 1;
  // 分母用板上座位数而非 status.total：后者来自服务端最新对局，可能与当前房间板子不一致
  const alive = players.length - dead.size;
  const stagePlayers = players.map((p) => ({
    ...p,
    status: dead.has(p.id) ? ('dead' as const) : p.status,
  }));
  const speaker =
    talk?.seat !== null && talk?.seat !== undefined
      ? players.find((p) => p.id === talk.seat)
      : undefined;
  return {
    phase: night ? ('night' as const) : ('day' as const),
    phaseLabel:
      last?.kind === GameEventKind.Over
        ? '🏁 对局结束'
        : night
          ? `🌙 第 ${day} 夜`
          : `☀️ 第 ${day} 天`,
    caption: `${roomName} · 第 ${day} ${night ? '夜' : '天'} · 存活 ${alive}/${players.length}`,
    stagePlayers,
    speaker,
    wolfChat,
    speechText: talk?.text,
    spotlightSeats: spotlightSeat !== null ? [spotlightSeat] : [],
    hostMessage: hostSay(last, night, { next, wolfTarget }),
  };
}

/**
 * 主持人口播：指令式标准主持词，无修饰。
 * seatName 需要 in-game 名字时由 name 参数补齐。
 */
export function hostSay(
  event: GameEventDto | undefined,
  isNight = false,
  /** 预读信息：next=未播的下一条（夜行动作间衔接「X 请闭眼」）；wolfTarget=刀口（女巫报死讯） */
  peek?: { next?: GameEventDto; wolfTarget?: string },
): string {
  if (!event) return '各位稍作等待，对局马上开始。';
  switch (event.kind) {
    case GameEventKind.Started:
      return '各位选手请入座。';
    case GameEventKind.Night:
      return '天黑请闭眼。';
    case GameEventKind.WolfChat:
      return '狼人请睁眼。请商量今晚袭击的玩家。';
    case GameEventKind.WolfPass:
      // 狼聊结束标志；「狼人请闭眼」由紧随的狼刀夜行动作播，这里不重复
      return '';
    case GameEventKind.NightAction: {
      // 服务端 text 自带角色名（如「4号（狼）：狼刀7号」）→ 按角色取完整主持词。
      // 狼的「请睁眼」已由狼聊事件播过，这里只负责「狼人请闭眼」收尾；
      // 其余角色整句含睁眼+闭眼，与相邻动作串成标准主持流。
      const role = /（(.+?)）/.exec(event.text)?.[1] ?? '';
      const lines: Record<string, string> = {
        狼: '狼人请闭眼。',
        预言家: '预言家请睁眼。请查验一名玩家的身份。预言家请闭眼。',
        守卫: '守卫请睁眼。请选择今晚守护的玩家。守卫请闭眼。',
        女巫:
          peek?.wolfTarget !== undefined
            ? `女巫请睁眼。昨晚 ${peek.wolfTarget} 号玩家出局，是否使用解药？是否使用毒药？女巫请闭眼。`
            : '女巫请睁眼。昨晚是平安夜。是否使用毒药？女巫请闭眼。',
      };
      return lines[role] ?? event.text; // 未知角色：原文兜底
    }
    case GameEventKind.Dawn:
      return event.deaths && event.deaths.length > 0
        ? `天亮请睁眼。昨夜 ${event.deaths.join('、')} 号玩家出局。`
        : '天亮请睁眼。昨夜是平安夜。';
    case GameEventKind.Speech:
      return event.seat !== null ? `${event.seat}号玩家请发言` : '请各位依次发言。';
    case GameEventKind.Vote:
      return event.tie
        ? '发言到此为止，请全体投票。平票，无人出局。'
        : `发言到此为止，请全体投票。投票结果：${event.target ?? ''}号玩家出局，请留遗言。`;
    case GameEventKind.Duel:
      return `平票，进入决斗环节。决斗结果：${event.target ?? ''}号玩家出局。`;
    case GameEventKind.Idiot:
      return `${event.seat ?? ''}号玩家是白痴，翻牌免死，失去投票权。`;
    case GameEventKind.Hunter:
      return event.target !== undefined
        ? `${event.seat ?? ''}号玩家的身份是猎人。枪响，带走 ${event.target} 号玩家。`
        : `${event.seat ?? ''}号玩家的身份是猎人。他选择放弃开枪。`;
    case GameEventKind.Died:
      // 夜晚死讯由 dawn 统一揭晓；白天出局（枪杀/决斗）由对应事件词承担，这里兜底简报
      return isNight ? '' : `${event.seat ?? ''}号玩家出局。`;
    case GameEventKind.Over:
      return `对局结束，${event.text}！`;
    default:
      return event.text;
  }
}
