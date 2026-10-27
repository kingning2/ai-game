import { Fragment, useMemo, type ComponentProps, type ReactNode } from 'react';
import ReactMarkdown, { type Components } from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { cn } from '../cn';

/**
 * 流式 Markdown 渲染（AI 输出场景）：
 * - react-markdown + remark-gfm：AST 组件化渲染，默认不执行原生 HTML（防 XSS）
 * - 默认主题（Apple HIG 令牌）：表格、代码块、引用、列表等全部可经 components 覆盖
 * - 流式稳定化：SSE 未收完时 fence 未闭合会导致后半文被吞进代码块，
 *   hook 自动补闭合（``` 与 ~~~ 分开计数），渲染结果在 chunk 间保持稳定
 */

export type HastNodeLike = {
  position?: {
    start?: { line?: number };
    end?: { line?: number };
  };
};

type CodeProps = ComponentProps<'code'> & { node?: HastNodeLike };

/** 行内 code 与块级 code 区分：块级（fence）跨行或带 language- class */
function CodeRenderer({ className, children, node }: CodeProps) {
  const start = node?.position?.start?.line;
  const end = node?.position?.end?.line;
  const isBlock =
    (start !== undefined && end !== undefined && start !== end) ||
    !!className?.includes('language-');
  if (isBlock) return <code className="font-mono">{children}</code>;
  return (
    <code className="rounded-md bg-tint/10 px-1 py-0.5 font-mono text-[0.85em] text-tint">
      {children}
    </code>
  );
}

/** 默认主题：全部元素可被 options.components 同名覆盖 */
export const markdownComponents: Components = {
  h1: ({ children }) => <h1 className="mb-2 mt-6 text-xl font-bold">{children}</h1>,
  h2: ({ children }) => <h2 className="mb-2 mt-6 text-lg font-semibold">{children}</h2>,
  h3: ({ children }) => <h3 className="mb-1.5 mt-5 text-base font-semibold">{children}</h3>,
  h4: ({ children }) => <h4 className="mb-1.5 mt-4 text-sm font-semibold">{children}</h4>,
  h5: ({ children }) => <h5 className="mb-1.5 mt-4 text-sm font-semibold">{children}</h5>,
  h6: ({ children }) => (
    <h6 className="mb-1.5 mt-4 text-sm font-semibold text-secondary-label">{children}</h6>
  ),
  p: ({ children }) => <p className="my-2">{children}</p>,
  a: ({ children, href }) => (
    <a
      href={href}
      target="_blank"
      rel="noreferrer"
      className="text-tint underline decoration-tint/40 underline-offset-2 hover:decoration-tint"
    >
      {children}
    </a>
  ),
  strong: ({ children }) => <strong className="font-semibold">{children}</strong>,
  del: ({ children }) => <del className="opacity-60">{children}</del>,
  ul: ({ children }) => <ul className="my-2 list-disc space-y-1 ps-5">{children}</ul>,
  ol: ({ children }) => <ol className="my-2 list-decimal space-y-1 ps-5">{children}</ol>,
  li: ({ children }) => <li className="ps-1">{children}</li>,
  blockquote: ({ children }) => (
    <blockquote className="my-3 rounded-e-xl border-s-[3px] border-tint/40 bg-tint/5 py-1 ps-3 [&>p]:my-1">
      {children}
    </blockquote>
  ),
  hr: () => <hr className="my-4 border-t border-separator" />,
  pre: ({ children }) => (
    <pre className="my-3 overflow-x-auto rounded-xl border border-separator bg-tertiary-fill p-3 font-mono text-[12.5px] leading-relaxed">
      {children}
    </pre>
  ),
  code: CodeRenderer,
  // 表格：圆角描边容器 + 横向滚动 + 表头填充 + 偶数行斑马纹
  table: ({ children }) => (
    <div className="my-3 overflow-x-auto rounded-xl border border-separator">
      <table className="w-full border-collapse text-[13px]">{children}</table>
    </div>
  ),
  th: ({ children }) => (
    <th className="border-b border-separator bg-tertiary-fill px-3 py-1.5 text-start font-semibold">
      {children}
    </th>
  ),
  td: ({ children }) => (
    <td className="border-b border-separator px-3 py-1.5 align-top">{children}</td>
  ),
  tr: ({ children }) => (
    <tr className="[&:last-child>td]:border-b-0 [&:nth-child(even)>td]:bg-tertiary-fill/50">
      {children}
    </tr>
  ),
  img: ({ src, alt }) => (
    <img
      src={src}
      alt={alt}
      className="my-3 max-w-full rounded-xl border border-separator"
      loading="lazy"
    />
  ),
};

export type UseMarkdownOptions = {
  /** 覆盖/扩展元素渲染（如自定义表格），与默认主题按 key 合并，调用方优先 */
  components?: Components;
  /** 跳过默认主题，完全使用传入的 components（默认 false） */
  unstyled?: boolean;
  /** 流式稳定化：为未闭合 fence 补闭合（默认 true） */
  stabilize?: boolean;
  /** 附加到容器上的 className */
  className?: string;
};

/** SSE 流里 fence 未闭合时补上闭合标记，避免未收完的代码块吞掉后续文本 */
export function stabilizeStreaming(src: string): string {
  const backtick = (src.match(/^ {0,3}(?:```+)/gm)?.length ?? 0) % 2 === 1;
  const tilde = (src.match(/^ {0,3}(?:~~~+)/gm)?.length ?? 0) % 2 === 1;
  let out = src;
  if (backtick) out += '\n```';
  if (tilde) out += '\n~~~';
  return out;
}

/**
 * 把 markdown 文本渲染成 ReactNode（内置默认主题，样式可逐元素覆盖）。
 * 流式场景：text 传「已接收的累计文本」，chunk 到达追加后 hook 自动重渲染。
 */
export function useMarkdown(text: string, options?: UseMarkdownOptions): ReactNode {
  const { components, unstyled = false, stabilize = true, className } = options ?? {};
  const merged = useMemo<Components>(
    () => (unstyled ? (components ?? {}) : { ...markdownComponents, ...components }),
    [unstyled, components],
  );
  return useMemo(() => {
    const src = stabilize ? stabilizeStreaming(text) : text;
    return (
      <div
        className={cn(
          'text-sm leading-relaxed text-label',
          '[&>*:first-child]:mt-0 [&>*:last-child]:mb-0',
          className,
        )}
      >
        <ReactMarkdown remarkPlugins={[remarkGfm]} components={merged}>
          {src}
        </ReactMarkdown>
      </div>
    );
  }, [text, merged, stabilize, className]);
}

/** <Markdown /> 薄壳：不写 hook 的场景直接传文本渲染 */
export function Markdown({ text, options }: { text: string; options?: UseMarkdownOptions }) {
  const node = useMarkdown(text, options);
  return <Fragment>{node}</Fragment>;
}
