
/**
 * 字典数据 Hook 模块
 * 
 * 提供类型安全的字典数据获取功能，支持：
 * - 多字典并行加载
 * - 类型推导和智能提示
 * - Suspense 模式
 * - 错误处理
 */

import { DictList } from '@shared/types/apis-dict';
import { ApiResult } from '@shared/types/apis-rest';
import { useQueries } from '@tanstack/react-query';
import { DictQueryContext, DictTypeMap, dictQueryKeyMap, dictQueryMap, paramParse } from './dict-data';
/**
 * 将 API 响应类型转换为运行时字典类型的工具类型
 * 将数组类型转换为 DictList 实例
 */
type ApiResponseToDictData<T> = {
  [K in keyof T]: T[K] extends any[] ? DictList : T[K];
};

/**
 * 将联合类型转换为交集类型的工具类型
 * 用于合并多个字典的字段到单一对象类型
 */
type UnionToIntersection<U> =
  (U extends any ? (_k: U) => void : never) extends
  ((_k: infer I) => void) ? I : never;

/**
 * 类型安全的字典数据类型
 * 根据传入的字典键数组推导出合并后的字典数据结构
 */
export type TypedDictData<T extends readonly (keyof DictTypeMap)[]> =
  UnionToIntersection<
    T[number] extends keyof DictTypeMap
    ? ApiResponseToDictData<DictTypeMap[T[number]]>
    : never
  >;

/**
 * 字典数据获取 Hook
 * 
 * @template T - 字典键的只读数组类型
 * @param dicts - 需要加载的字典键数组
 * @param params - 查询参数，可以是对象或任意类型
 * @returns 包含字典数据、加载状态、错误信息等的对象
 * 
 * @example
 * ```typescript
 * // 加载单个字典
 * const { dictData } = useDictData(['app'] as const);
 * console.log(dictData.app_status.getLabel('1')); // 获取状态标签
 * 
 * // 加载多个字典
 * const { dictData } = useDictData(['app', 'rbac'] as const);
 * console.log(dictData.app_status.getLabel('1')); // app 字典字段
 * console.log(dictData.audit_result.getLabel('pass')); // rbac 字典字段
 * 
 * // 带参数加载
 * const { dictData } = useDictData(['app'] as const, { version: '1.0' });
 * ```
 */



export function useDictData<T extends readonly (keyof DictTypeMap)[]>(
  dicts: T,
  params?: Record<string, any> | any
): {
  readonly dictData: TypedDictData<T>;
  readonly isLoading: boolean;
  readonly isError: boolean;
  readonly errors: any[];
  readonly refetch: () => void;
} {
  const results = useQueries({
    queries:
      dicts.map((key) => {
        // 参数处理逻辑
        const param = paramParse(key, params);

        // 获取字典查询函数和 queryKey 生成函数
        const queryFn = dictQueryMap[key as keyof DictTypeMap];
        const queryKeyFn = dictQueryKeyMap[key as keyof DictTypeMap];

        if (!queryFn) {
          throw new Error(`No dict query function found for key: ${String(key)}`);
        }
        if (!queryKeyFn) {
          throw new Error(`No dict queryKey function found for key: ${String(key)}`);
        }

        return {
          queryKey: queryKeyFn(param),
          queryFn: async (context: DictQueryContext) => {
            return await queryFn(context, param);
          }
        };
      }),
  });

  // 处理加载状态
  const isLoading = results.some((result) => result.isLoading);

  // 将结果转换为字典数据对象
  const [dictData, isError, errors] = (() => {
    const data = {} as any;

    // 处理每个查询结果
    results.forEach((result) => {
      const apiResult = result.data as ApiResult<any>;

      if (apiResult?.status && apiResult?.response) {
        const response = apiResult.response;
        // 将 API 响应中的字典数组转换为 DictList 实例
        // response 结构如 { app_status: [...], request_status: [...], ... }
        Object.entries(response).forEach(([fieldName, value]) => {
          if (Array.isArray(value)) {
            data[fieldName] = Object.assign(new DictList(), value);
          }
        });
      }
    });

    // 处理错误状态
    const isError = results.some((result) =>
      result.isError || (!result.isLoading && result.data && !result.data.status)
    );


    const errors = results.map((result) => {
      if (result.isError || result.isLoading) {
        return result.error;
      }
      if (result.data && !result.data.status) {
        return new Error(result.data?.message || 'Unknown error');
      }
      return null;
    }).filter(Boolean);

    return [data as TypedDictData<T>, isError, errors];
  })();

  // 重试方法：重新获取所有字典数据
  const refetch = () => {
    results.forEach(result => result.refetch());
  };

  return {
    dictData,
    isLoading,
    isError,
    errors,
    refetch
  };
}
