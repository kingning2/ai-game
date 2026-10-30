/**
 * repo-guard — agent-packages 项目守卫（pi 扩展）
 *
 * 三个 hook，把 AGENTS.md 的口头约束变成机械强制：
 * 1. 硬编码色值拦截：edit/write 写入 packages|apps 下的 .ts/.tsx/.css（tokens.css 豁免）
 *    时扫描新增内容，发现 hex/rgb/hsl 色值、dark: 主题分支、white/black 工具类直接 block。
 * 2. rules 自动注入：按触达文件所属领域，把 rules/*.md 内容追加进对应工具结果（每会话每文件一次）。
 * 3. 验收提醒：本轮改过代码但没跑 pnpm build/typecheck，agent_end 时提醒。
 *
 * 误报逃生舱：本次写入内容任意位置含 `token-guard:allow` 则跳过色值扫描。
 * 安装：项目 .pi/extensions/，下个会话生效。
 */
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { ExtensionAPI, ToolCallEvent } from '@earendil-works/pi-coding-agent';

const ALLOW_MARK = 'token-guard:allow';
const RULE_MAX_CHARS = 6000;

/** 色值违规模式：与 rules/web/ui-design.md「零硬编码」条款一一对应 */
const COLOR_RULES: Array<[RegExp, string]> = [
  [/#[0-9a-fA-F]{3}|#[0-9a-fA-F]{4}|#[0-9a-fA-F]{6}|#[0-9a-fA-F]{8}\b/g, 'hex 色值'],
  [/rgba?\(/g, 'rgb/rgba()'],
  [/hsla?\(/g, 'hsl/hsla()'],
  [/\bdark:/g, 'dark: 主题分支（明暗差异必须走令牌）'],
  [/\b(?:bg|text|border|ring|fill|stroke|shadow|outline)-(?:white|black)\b/g, 'white/black 工具类'],
];

/** packages|apps 下的源码/样式文件 → 需要守卫；tokens.css 与 node_modules 豁免 */
function guardedKind(path: string): 'code' | 'style' | null {
  const p = path.replaceAll('\\', '/');
  if (p.includes('node_modules') || p.includes('tokens.css')) return null;
  if (!/(^|\/)(packages|apps)\//.test(p)) return null;
  if (p.endsWith('.ts') || p.endsWith('.tsx')) return 'code';
  if (p.endsWith('.css')) return 'style';
  return null;
}

/** 取本次调用真正新写入的文本 */
function newTextOf(event: ToolCallEvent): string {
  if (event.toolName === 'write') return event.input.content;
  if (event.toolName === 'edit') return event.input.edits.map((e) => e.newText).join('\n');
  return '';
}

function findViolations(text: string): string[] {
  const found = new Set<string>();
  for (const [pattern, label] of COLOR_RULES) {
    pattern.lastIndex = 0;
    for (const m of text.matchAll(pattern)) {
      found.add(`${label}: ${m[0]}`);
      if (found.size >= 5) return [...found];
    }
  }
  return [...found];
}

export default function repoGuard(pi: ExtensionAPI) {
  let dirtyAt: number | null = null; // 本轮首次改代码的时间；null = 已验收或无改动
  const injected = new Set<string>(); // 本会话已注入过的规则文件

  // ── 1. 硬编码色值拦截 ─────────────────────────────────────────────
  pi.on('tool_call', async (event) => {
    if (event.toolName !== 'edit' && event.toolName !== 'write') return;
    if (!guardedKind(event.input.path)) return;

    const text = newTextOf(event);
    if (!text || text.includes(ALLOW_MARK)) return;

    const violations = findViolations(text);
    if (violations.length === 0) return;

    return {
      block: true,
      reason:
        `repo-guard：检测到硬编码色值/主题分支（AGENTS.md 核心约束 #3）——\n` +
        violations.map((v) => `  · ${v}`).join('\n') +
        `\n颜色、阴影、缓动一律走 packages/shared-ui/styles/tokens.css 令牌，详见 rules/web/ui-design.md。` +
        `\n明暗两套都必须可读；确属误报（如锚点 id）可在本次写入内容任意处加 ${ALLOW_MARK} 重试。`,
    };
  });

  // ── 2. rules 自动注入 + 3. 改动标记 ───────────────────────────────
  pi.on('tool_result', async (event, ctx) => {
    if (event.isError) return;

    if (event.toolName === 'bash' || event.toolName === 'powershell') {
      const cmd = (event.input as { command?: string }).command ?? '';
      if (/pnpm\s+(build|typecheck)/.test(cmd)) dirtyAt = null;
      return;
    }

    if (event.toolName !== 'edit' && event.toolName !== 'write') return;

    const kind = guardedKind(event.input.path);
    if (!kind) return;
    dirtyAt ??= Date.now();

    // 按触达文件选规则：tsx → UI 双件套；css → ui-design；ts → coding-style；新建文件 → 另加 project-structure
    const rules: string[] = [];
    if (kind === 'code' && event.input.path.endsWith('.tsx')) {
      rules.push('rules/web/ui-design.md', 'rules/web/component-guide.md');
    } else if (kind === 'style') {
      rules.push('rules/web/ui-design.md');
    } else {
      rules.push('rules/common/coding-style.md');
    }
    if (event.toolName === 'write') rules.push('rules/common/project-structure.md');

    for (const rule of rules) {
      if (injected.has(rule)) continue;
      injected.add(rule);
      try {
        // ponytail: 同步读，单文件 <10KB，注入频率每会话一次，不值得异步化
        const content = readFileSync(join(ctx.cwd, rule), 'utf-8');
        event.content.push({
          type: 'text',
          text:
            `\n\n---\n[repo-guard] 触达 ${event.input.path}，自动注入 ${rule}：\n` +
            content.slice(0, RULE_MAX_CHARS),
        });
      } catch {
        injected.delete(rule); // 文件缺失不阻塞，下次再试
      }
    }
  });

  // ── 3. 验收提醒 ───────────────────────────────────────────────────
  pi.on('agent_end', async (_event, ctx) => {
    if (dirtyAt === null || !ctx.hasUI) return;
    ctx.ui.notify(
      'repo-guard：本轮有代码改动但未跑 pnpm build && pnpm typecheck（交付验收 #1）',
      'warning',
    );
  });

  pi.on('session_start', async () => {
    dirtyAt = null;
    injected.clear();
  });
}
