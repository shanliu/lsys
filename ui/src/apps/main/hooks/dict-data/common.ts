/**
 * 字典系统通用类型定义
 * 
 * 包含字典系统的基础类型，避免循环引用
 */

import { ApiResult } from '@shared/types/apis-rest';
import type { QueryFunctionContext } from '@tanstack/react-query';

export function paramParse<P = any>(key: string, params?: Record<string, P> | P): P | Record<string, P> | undefined {
  // 保留原有逻辑，添加具体输入输出类型以便 TypeScript 推断
  if (params === undefined) {
    return undefined;
  }

  // 当 params 是对象且不是数组时，优先使用 params[key]（如果存在），否则返回整个对象作为参数
  if (typeof params === 'object' && !Array.isArray(params)) {
    return Object.prototype.hasOwnProperty.call(params, key) ? (params as Record<string, P>)[key] : params as Record<string, P>;
  }

  // 其他类型（标量/数组），直接返回
  return params as P;
}

/**
 * React Query 查询上下文类型
 * 用于字典查询函数的上下文参数
 */
export type DictQueryContext = QueryFunctionContext<readonly [string, ...unknown[]]>;

/**
 * 字典查询函数类型
 * @template P - 参数类型
 * @template R - 响应类型
 */
export type DictQueryFn<P = any, R = any> = (
  _context: DictQueryContext,
  _params?: P
) => Promise<ApiResult<R>>;
