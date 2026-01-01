import type { NavigateOptions } from "@tanstack/react-router";
import { useCallback } from "react";

/**
 * 创建搜索导航 Hook
 * 用于合并当前搜索参数并导航
 *
 * @param navigate - TanStack Router 的 navigate 函数
 * @param currentSearch - 当前的搜索参数对象
 * @returns 搜索导航函数
 *
 * @example
 * ```tsx
 * const navigate = useNavigate()
 * const filterParam = useSearch({ strict: false })
 *
 * const searchGo = useSearchNavigate(navigate, filterParam)
 *
 * // 使用
 * searchGo({ status: 1, page: 2 }) // 会合并当前的搜索参数
 * ```
 */
export function useSearchNavigate<TSearchParams extends Record<string, any>>(
  navigate: (opts: NavigateOptions) => Promise<void>,
  currentSearch: TSearchParams,
) {
  return useCallback(
    (params: Partial<TSearchParams>) => {
      navigate({
        search: {
          ...currentSearch,
          ...params,
        } as any,
      });
    },
    [navigate, currentSearch],
  );
}
