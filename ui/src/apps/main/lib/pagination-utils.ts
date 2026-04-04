import type { TotalRecrodResType } from "@shared/types/base-schema";
import type { NavigateOptions } from "@tanstack/react-router";
import { useCallback, useRef } from "react";

// Re-export TotalRecrodResType from shared for convenience
export type { TotalRecrodResType } from "@shared/types/base-schema";

// Re-export UI components from shared for convenience
export {
    CursorPagination,
    PagePagination,
    type CursorPaginationProps,
    type PagePaginationProps
} from '@shared/components/custom/pagination';

/**
 * 可选的分页数选项（用于分页下拉选择）
 */
export const PAGE_SIZE_OPTIONS = [20, 50, 100] as const;

/**
 * 默认分页数
 */
export const DEFAULT_PAGE_SIZE = 20;

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

/**
 * Limit（游标）分页的 count_num 优化 Hook
 *
 * 优化策略：
 * 1. 初始化时 count_num = true，首次请求统计总数
 * 2. 加载到 total > 0 后设为 false，后续翻页不再统计
 * 3. 当 cursor.next 和 cursor.prev 均为 null（无更多数据）时重置为 true
 * 4. 传入 filters 后自动监听变化并重置
 *
 * @example
 * ```tsx
 * const countNum = useLimitCountNum(filters);
 *
 * const query = useQuery({
 *   queryKey: ["data", params],
 *   queryFn: () => fetchData({ ...params, count_num: countNum.getCountNum() }),
 * });
 *
 * query.isSuccess && countNum.handleQueryResult(query.data);
 * const totalInfo = countNum.getTotalInfo();
 * ```
 */
export function useLimitCountNum(filters?: Record<string, any>) {
    const countNumRef = useRef(true);
    const totalInfoRef = useRef<TotalRecrodResType | null>(null);
    const hasLoadedRef = useRef(false);
    const lastFiltersRef = useRef<string>(JSON.stringify(filters || {}));

    const currentFilters = JSON.stringify(filters || {});
    if (filters && currentFilters !== lastFiltersRef.current) {
        countNumRef.current = true;
        hasLoadedRef.current = false;
        lastFiltersRef.current = currentFilters;
    }

    return {
        getCountNum: useCallback(() => countNumRef.current, []),
        getTotalInfo: useCallback((): TotalRecrodResType | null => totalInfoRef.current, []),
        hasTotalInfo: useCallback(() => (totalInfoRef.current?.exact ?? totalInfoRef.current?.over ?? 0) > 0, []),

        handleQueryResult: useCallback((queryData: {
            response?: {
                total?: { exact?: number | null; over?: number | null } | null;
                cursor?: { next?: number | null; prev?: number | null } | null;
            };
        }) => {
            if (queryData?.response) {
                const { total, cursor } = queryData.response;

                if (total) {
                    totalInfoRef.current = total;
                }

                const numTotal = total?.exact ?? total?.over ?? null;

                if (numTotal !== null && numTotal >= 0) {
                    hasLoadedRef.current = true;
                    countNumRef.current = false;
                }

                const next_cursor = cursor?.next;
                const prev_cursor = cursor?.prev;

                if (hasLoadedRef.current && (next_cursor === null || next_cursor === undefined) && (prev_cursor === null || prev_cursor === undefined)) {
                    countNumRef.current = true;
                }
            }
        }, []),

        reset: useCallback(() => {
            countNumRef.current = true;
            hasLoadedRef.current = false;
            totalInfoRef.current = null;
        }, []),



    };
}

/**
 * Page（页码）分页的 count_num 优化 Hook
 *
 * 优化策略：
 * 1. 初始化时 count_num = true，首次请求统计总数
 * 2. 加载到 total > 0 后设为 false，后续翻页不再统计
 * 3. 当 currentPage 接近最后一页时重置为 true
 * 4. 传入 filters 后自动监听变化并重置
 *
 * @example
 * ```tsx
 * const countNum = usePageCountNum(filters);
 *
 * const query = useQuery({
 *   queryKey: ["data", params],
 *   queryFn: () => fetchData({ ...params, count_num: countNum.getCountNum() }),
 * });
 *
 * query.isSuccess && countNum.handleQueryResult(query.data, currentPage, pageSize);
 * const total = countNum.getTotal();
 * ```
 */
export function usePageCountNum(filters?: Record<string, any>) {
    const countNumRef = useRef(true);
    const totalRef = useRef<number | null>(null);
    const hasLoadedRef = useRef(false);
    const lastFiltersRef = useRef<string>(JSON.stringify(filters || {}));

    const currentFilters = JSON.stringify(filters || {});
    if (filters && currentFilters !== lastFiltersRef.current) {
        countNumRef.current = true;
        hasLoadedRef.current = false;
        lastFiltersRef.current = currentFilters;
    }

    return {
        getCountNum: useCallback(() => countNumRef.current, []),
        getTotal: useCallback(() => totalRef.current, []),
        hasTotal: useCallback(() => (totalRef.current ?? 0) > 0, []),

        handleQueryResult: useCallback((
            queryData?: {
                response?: {
                    total?: number | null;
                    count?: number | null;
                    [key: string]: unknown;
                };
            },
            currentPage?: number,
            pageSize?: number
        ) => {
            if (queryData?.response) {
                const total = queryData.response.total ?? queryData.response.count ?? null;

                if (total !== null && total >= 0) {
                    totalRef.current = total;
                    hasLoadedRef.current = true;
                    countNumRef.current = false;

                    if (currentPage !== undefined && pageSize !== undefined && pageSize > 0) {
                        const totalPages = Math.ceil(total / pageSize);
                        if (currentPage >= totalPages - 1) {
                            countNumRef.current = true;
                        }
                    }
                }
            }
        }, []),

        reset: useCallback(() => {
            countNumRef.current = true;
            hasLoadedRef.current = false;
            totalRef.current = null;
        }, []),



    };
}
