import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * 获取浮点数计算差异指纹
 * @returns 浮点数计算特征数组
 */
function getFloatCalculationFingerprint(): string[] {
  const results: string[] = [];

  // 测试各种浮点数运算的精度差异
  const testValues = [
    0.1 + 0.2, // 经典的浮点数精度问题
    Math.PI, // 圆周率
    Math.E, // 自然对数底数
    Math.sqrt(2), // 根号2
    Math.sin(Math.PI / 4), // 三角函数
    Math.cos(Math.PI / 3),
    Math.tan(Math.PI / 6),
    Math.log(2), // 对数运算
    Math.exp(1),
    1 / 3, // 简单除法
    2 / 3,
    7 / 9,
    Math.pow(2, 0.5), // 幂运算
    Math.pow(3, 1 / 3),
    Math.atan2(1, 1), // 反三角函数
    Math.asin(0.5),
    Math.acos(0.5),
  ];

  // 执行更复杂的计算组合
  const complexCalculations = [
    Math.sin(1) * Math.cos(1),
    Math.sqrt(Math.PI) + Math.E,
    (0.1 + 0.2) * 3,
    Math.log(Math.PI) / Math.log(2),
    Math.pow(Math.E, Math.PI) - Math.pow(Math.PI, Math.E),
    Math.sin(Math.sqrt(2)) + Math.cos(Math.sqrt(3)),
  ];

  // 将所有计算结果转为高精度字符串
  const allValues = [...testValues, ...complexCalculations];
  allValues.forEach((value) => {
    results.push(value.toString());
    results.push(value.toPrecision(15)); // 15位精度
    results.push(value.toFixed(10)); // 10位小数
  });

  // 测试 Date 对象的精度
  const now = Date.now();
  results.push((now * 0.1).toString());
  results.push((now / 7).toString());

  // 测试 performance.now() 的精度（如果可用）
  if (typeof performance !== "undefined" && performance.now) {
    const perfNow = performance.now();
    results.push(perfNow.toString());
    results.push((perfNow * Math.PI).toString());
  }

  return results;
}

/**
 * 获取浏览器指纹
 * @returns 浏览器指纹字符串
 */
function getBrowserFingerprint(): string {
  // 只使用浮点数计算差异指纹
  const features = getFloatCalculationFingerprint();

  // 将所有特征连接成字符串
  const fingerprintString = features.join("|");

  // 使用简单的哈希算法生成指纹
  let hash = 0;
  for (let i = 0; i < fingerprintString.length; i++) {
    const char = fingerprintString.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash; // 转换为32位整数
  }

  return Math.abs(hash).toString(36);
}

/**
 * 基于浏览器指纹生成确定性的随机字符串
 * @param length 字符串长度
 * @param seed 可选的种子值，用于在同一指纹下生成不同的字符串
 * @returns 基于指纹的随机字符串
 */
export function generateRandomString(length: number, seed?: string): string {
  const chars =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  const fingerprint = getBrowserFingerprint();
  const seedString = seed ? `${fingerprint}_${seed}` : fingerprint;

  // 基于指纹和种子创建伪随机数生成器
  let hash = 0;
  for (let i = 0; i < seedString.length; i++) {
    const char = seedString.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash;
  }

  // 使用线性同余生成器创建确定性随机序列
  let seed_value = Math.abs(hash);
  const a = 1664525;
  const c = 1013904223;
  const m = Math.pow(2, 32);

  let result = "";
  for (let i = 0; i < length; i++) {
    seed_value = (a * seed_value + c) % m;
    const index = Math.floor((seed_value / m) * chars.length);
    result += chars.charAt(index);
  }

  return result;
}

/**
 * 从 useQuery 返回结果中提取响应数据
 * 支持列表数据和单个对象数据
 *
 * @param queryData useQuery 返回的结果对象
 * @param defaultValue 默认值（必填），如果数据不存在则返回此值
 * @returns 响应数据或默认值
 */
export function getQueryResponseData<T>(
  queryData: { response?: { data?: T } } | any,
  defaultValue: T,
): T {
  return queryData?.response?.data ?? defaultValue;
}

/**
 * 从 useQuery 返回结果中提取响应的 next 字段（用于偏移分页）
 *
 * @param queryData useQuery 返回的结果对象
 * @returns next 字段值或 null
 */
export function getQueryResponseNext(
  queryData: { response?: { next?: number | null } } | any,
): number | null {
  return queryData?.response?.next ?? null;
}

/**
 * 从数组中提取指定字段的最小值和最大值
 * 
 * @param array - 对象数组
 * @param field - 要提取值的字段名
 * @param minKey - 返回对象中最小值的键名
 * @param maxKey - 返回对象中最大值的键名
 * @returns 包含最小值和最大值的对象
 */
export function extractMinMax<
  T extends Record<string, any>,
  K extends keyof T,
  MinKey extends string,
  MaxKey extends string
>(
  array: T[],
  field: K,
  minKey: MinKey,
  maxKey: MaxKey
): { [P in MinKey | MaxKey]: number | null } {
  if (array.length === 0) {
    return {
      [minKey]: null,
      [maxKey]: null,
    } as { [P in MinKey | MaxKey]: number | null };
  }

  const values = array.map(item => item[field]);
  const minValue = Math.min(...values as number[]);
  const maxValue = Math.max(...values as number[]);

  return {
    [minKey]: minValue,
    [maxKey]: maxValue,
  } as { [P in MinKey | MaxKey]: number | null };
}

/**
 * 验证是否为有效的域名
 * @param domain - 域名字符串
 * @param allowSubdomain - 是否允许子域名，默认true
 * @returns 是否为有效域名
 */
export function isDomain(domain: string, allowSubdomain: boolean = true): boolean {
  if (!domain || typeof domain !== 'string') return false;
  const trimmed = domain.trim();

  if (allowSubdomain) {
    const domainRegex = /^([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)*[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.([a-zA-Z]{2,})$/;
    return domainRegex.test(trimmed);
  } else {
    const domainRegex = /^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.([a-zA-Z]{2,})$/;
    return domainRegex.test(trimmed);
  }
}

/**
 * 计算剩余秒数
 */
export function calculateRemainingSeconds(timeOut: Date | null): number {
  if (timeOut === null) {
    return 0
  }
  const now = Date.now()
  const remaining = Math.floor((timeOut.getTime() - now) / 1000)
  return remaining > 0 ? remaining : 0
}

/**
 * 获取首页 URL
 * @returns 首页 URL 字符串
 */
export function getHomeUrl(): string {
  return '/index.html'
}
