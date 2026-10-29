/**
 * 基础类型判别（is*）工具 —— 全仓唯一出处，禁止任何包自造 typeof 判别或结构守卫。
 *
 * 规则（见 rules/common/package-rules.md）：
 * - 全仓禁用 unknown：判别方法入参一律用泛型 T 捕获调用方的实际类型；
 * - 收窄用交叉类型（`value is T & string`），不丢调用方的原始类型信息。
 */

/** 是否为字符串 */
export function isString<T>(value: T): value is T & string {
  return typeof value === 'string';
}

/** 是否为非空字符串（去首尾空白后仍有内容） */
export function isNonEmptyString<T>(value: T): value is T & string {
  return typeof value === 'string' && value.trim().length > 0;
}

/** 是否为数字（NaN / ±Infinity 也属于 number，要有限数请用 isFiniteNumber） */
export function isNumber<T>(value: T): value is T & number {
  return typeof value === 'number';
}

/** 是否为有限数（排除 NaN / ±Infinity） */
export function isFiniteNumber<T>(value: T): value is T & number {
  return typeof value === 'number' && Number.isFinite(value);
}

/** 是否为整数 */
export function isInteger<T>(value: T): value is T & number {
  return Number.isInteger(value);
}

/** 是否为布尔值 */
export function isBoolean<T>(value: T): value is T & boolean {
  return typeof value === 'boolean';
}

/** 是否为函数（收窄后仍保留调用方的具体函数类型） */
export function isFunction<T>(value: T): value is T & ((...args: never[]) => void) {
  return typeof value === 'function';
}

/** 是否为对象（非 null、非数组；收窄后保留调用方的具体类型） */
export function isObject<T extends object>(
  value: T | null | undefined,
): value is T {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/** 是否为数组（收窄后保留调用方的元素类型） */
export function isArray<T>(value: T): value is T & { length: number } {
  return Array.isArray(value);
}

/** 是否为 Symbol */
export function isSymbol<T>(value: T): value is T & symbol {
  return typeof value === 'symbol';
}

/** 是否为 null 或 undefined（收窄到 never：T 排除 null/undefined 后无可留类型） */
export function isNil<T>(value: T | null | undefined): value is Extract<T, null | undefined> {
  return value === null || value === undefined;
}

/** 是否为 null */
export function isNull<T>(value: T | null): value is Extract<T, null> {
  return value === null;
}

/** 是否为 undefined */
export function isUndefined<T>(value: T | undefined): value is Extract<T, undefined> {
  return value === undefined;
}
