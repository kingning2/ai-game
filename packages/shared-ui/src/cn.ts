/** 极简 class 合并：过滤空值后拼接（不引第三方依赖） */
export const cn = (...parts: Array<string | false | null | undefined>): string =>
  parts.filter(Boolean).join(' ');
