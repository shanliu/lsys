/**
 * 字典映射生成模块
 * 
 * 从字典定义中自动生成：
 * - DictTypeMap: 字典响应类型映射
 * - dictQueryMap: 查询函数映射
 */

import { dictDefinitions } from './dicts';

/**
 * 自动推导的字典类型映射
 * 从字典定义中提取所有响应类型
 */
export type DictTypeMap = {
  [K in keyof typeof dictDefinitions]: typeof dictDefinitions[K]['type'];
};

/**
 * 从字典定义中自动提取查询函数映射
 * 运行时使用的查询函数集合
 */
export const dictQueryMap = Object.fromEntries(
  Object.entries(dictDefinitions).map(([key, def]) => [key, def.queryFn])
) as {
  [K in keyof typeof dictDefinitions]: typeof dictDefinitions[K]['queryFn'];
};

/**
 * 从字典定义中自动提取 queryKey 映射
 * 运行时使用的 queryKey 生成函数集合
 */
export const dictQueryKeyMap = Object.fromEntries(
  Object.entries(dictDefinitions).map(([key, def]) => [key, def.queryKey])
) as {
  [K in keyof typeof dictDefinitions]: typeof dictDefinitions[K]['queryKey'];
};

// 重新导出通用类型，方便外部使用
export type { DictQueryContext } from './common';
