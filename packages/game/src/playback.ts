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
      return Math.min(2600 + event.text.length * 90, 8000);
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
  // 发言/狼聊：前缀里最近一条带文本的事件（含说话座位）。
  // 夜晚只在“最后一条事件就是这条发言”时展示：狼聊之后还有狼刀/女巫/预言家等夜行窗口，
  // 聚光灯已亮在行动者身上，发言栏再残留狼的耳语会像串场；白天发言按设计演完仍停留
  const talk = events
    .toReversed()
    .find(
      (e) =>
        (e.kind === GameEventKind.Speech || e.kind === GameEventKind.WolfChat) &&
        e.seat !== null &&
        !(night && e !== last),
    );
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
    phase: (night ? 'night' : 'day') as 'night' | 'day',
    phaseLabel:
      last?.kind === GameEventKind.Over ? '🏁 对局结束' : night ? `🌙 第 ${day} 夜` : `☀️ 第 ${day} 天`,
    caption: `${roomName} · 第 ${day} ${night ? '夜' : '天'} · 存活 ${alive}/${players.length}`,
    stagePlayers,
    speaker,
    speechText: talk?.text,
    spotlightSeats: spotlightSeat !== null ? [spotlightSeat] : [],
    hostMessage: hostSay(last, speaker?.name, night),
  };
}

/** 主持人口播：按事件类型给台词（seatName 需要 in-game 名字时由 name 参数补齐） */
export function hostSay(event: GameEventDto | undefined, name?: string, isNight = false): string {
  if (!event) return '各位稍作等待，主持人正在整理场上的局势……';
  const seatName = (n: number): string => `${n} 号玩家${name ? `（${name}）` : ''}`;
  switch (event.kind) {
    case GameEventKind.Started:
      return '各位选手，请入座。身份牌已经握在每个人的手心里——是深藏不露，还是主动搅局？故事，从今夜开始。';
    case GameEventKind.Night:
      return `天黑请闭眼——第 ${event.day} 夜拉开帷幕。夜色漫过村庄，所有人都沉沉睡去。好梦，还是噩梦，天亮才知道。`;
    case GameEventKind.NightAction:
      return '夜深了，有人悄悄睁开了眼睛，在黑暗中默默做出自己的选择……';
    case GameEventKind.WolfChat:
      return '狼人请睁眼——狼友们，压低声音商量一下：今晚，把谁送走？';
    case GameEventKind.WolfPass:
      return '狼人们已经心意已决。獠牙收起，静待黎明。';
    case GameEventKind.Dawn:
      return event.deaths && event.deaths.length > 0
        ? `天亮请睁眼——昨夜不是平安夜。${event.deaths.map((n) => seatName(n)).join('、')}，永远地留在了黑暗里。请默哀——然后擦亮眼睛，狼，就藏在你们中间。`
        : '天亮请睁眼——昨天是平安夜！昨夜无人出局。但别高兴得太早，狼的獠牙，今晚还会再来。';
    case GameEventKind.Speech:
      return event.seat !== null
        ? `有请 ${event.seat} 号玩家${name ? `，${name}` : ''}，请开始你的发言。`
        : '好，下面进入发言环节——请各位依次开始你的表演。';
    case GameEventKind.Vote:
      return event.tie
        ? '投票结果出炉——平票！今天无人被放逐，但怀疑的种子已经种下。'
        : event.target !== undefined
          ? `投票结果出炉——${event.target} 号玩家，被放逐出局。请留下你的遗言。`
          : '发言到此为止。现在，请把你手中的一票，投给你心中的狼人。';
    case GameEventKind.Duel:
      return event.target !== undefined
        ? `决斗见分晓——${event.target} 号玩家倒下了！生死，就在一念之间。`
        : '平票了！按照规则，进入决斗环节——生死，就在一念之间。';
    case GameEventKind.Idiot:
      return event.seat !== null
        ? `翻牌！${event.seat} 号竟然是白痴——他可以留在场上，但从今天起失去投票权。`
        : '翻牌！竟然是白痴——他可以留在场上，但从今天起失去投票权。';
    case GameEventKind.Hunter:
      return event.target !== undefined
        ? `猎人亮出了身份——枪响！${event.target} 号应声倒地。`
        : '猎人收起了枪——把子弹带进了坟墓，放弃开枪。';
    case GameEventKind.Died:
      // 夜晚遇害（天亮前的揭晓仪式）与白天出局（放逐/枪杀/决斗）口播不同
      if (isNight) {
        return event.seat !== null
          ? `夜色中传来噩耗——${event.seat} 号玩家出局了。请全体默哀，然后擦亮眼睛：狼，就藏在你们中间。`
          : '夜色中传来噩耗，有位选手永远地留在了黑暗里。请默哀——然后擦亮眼睛，狼，就藏在你们中间。';
      }
      return event.seat !== null
        ? `${event.seat} 号玩家出局。场上少了一个身影，真相却更近了一步。`
        : '有位选手永远地留在了黑暗里。请默哀——然后擦亮眼睛，狼，就藏在你们中间。';
    case GameEventKind.Over:
      return `本局结束！${event.text}。无论胜败，掌声送给每一位选手——夜幕终会再来，我们下次再会。`;
    default:
      return event.text;
  }
}
