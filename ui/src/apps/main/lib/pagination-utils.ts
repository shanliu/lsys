import type { NavigateOptions } from "@tanstack/react-router";
import { useCallback, useRef } from "react";

// Re-export UI components from shared for convenience
export {
    OffsetPagination,
    PagePagination,
    type OffsetPaginationProps,
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
 * 创建 count_num 优化管理器
 * 用于优化分页查询中的 count_num 参数，减少不必要的总数统计请求
 *
 * 优化策略：
 * 1. 初始化时 count_num = true，首次请求统计总数
 * 2. 加载到 total > 0 后设为 false，后续翻页不再统计
 * 3. Limit分页：当 next 为 null（无下一页）时重置为 true
 * 4. 筛选条件变化时需手动调用 reset() 重置为 true
 *
 * @example
 * ```tsx
 * // 1. 创建管理器
 * const countNumManager = useCountNumManager();
 *
 * // 2. 筛选条件变化时重置
 * useEffect(() => {
 *   countNumManager.reset();
 * }, [filters.field1, filters.field2]);
 *
 * // 3. 构建查询参数
 * const queryParams = {
 *   limit: pagination,
 *   count_num: countNumManager.getCountNum(),
 *   ...filters,
 * };
 *
 * // 4. 查询数据
 * const query = useQuery({
 *   queryKey: ["data", queryParams],
 *   queryFn: async () => await fetchData(queryParams)
 * });
 *
 * // 5. 处理 Limit 分页响应（自动提取 total 和 next）
 * query.isSuccess&&countNumManager.handleLimitQueryResult(query.data);;
 * 
 * // 或处理 Page 分页响应
 * query.isSuccess&&countNumManager.handlePageQueryResult(query.data);;
 *
 * // 6. 获取 total
 * const total = countNumManager.getTotal();
 * ```
 */
export function useCountNumManager(filters?: Record<string, any>) {
    const countNumRef = useRef(true);
    const totalRef = useRef<number | null>(null);
    const hasLoadedRef = useRef(false);
    const lastFiltersRef = useRef<string>(JSON.stringify(filters || {}));

    // 自动检测筛选条件变化并重置
    const currentFilters = JSON.stringify(filters || {});
    if (filters && currentFilters !== lastFiltersRef.current) {
        countNumRef.current = true;
        hasLoadedRef.current = false;
        lastFiltersRef.current = currentFilters;
    }

    return {
        /**
         * 获取当前的 count_num 值
         * @returns 是否需要统计总数
         */
        getCountNum: useCallback(() => {
            return countNumRef.current;
        }, []),

        /**
         * 获取缓存的 total 值
         * @returns 总数，如果未加载则返回 null
         */
        getTotal: useCallback(() => {
            return totalRef.current;
        }, []),

        /**
         * 处理 Limit 分页的查询结果
         * 自动提取 response.total 和 response.next_cursor/prev_cursor 并更新状态
         * 服务端直接返回 next_cursor 和 prev_cursor 字段
         * @param queryResult - TanStack Query 的查询结果对象
         */
        handleLimitQueryResult: useCallback((queryData: {
            response?: {
                total?: number | null;
                next_cursor?: number | null;
                prev_cursor?: number | null;
            };
        }) => {
            if (queryData?.response) {
                const { total, next_cursor, prev_cursor } = queryData.response;

                // 如果返回了有效的 total（包括0），缓存并设置 count_num = false
                if (total !== null && total !== undefined && total >= 0) {
                    totalRef.current = total;
                    hasLoadedRef.current = true;
                    countNumRef.current = false;
                }

                // 如果没有下一页游标，重置 count_num = true
                if (hasLoadedRef.current && (next_cursor === null || next_cursor === undefined) && (prev_cursor === null || prev_cursor === undefined)) {
                    countNumRef.current = true;
                }
            }
        }, []),

        /**
         * 处理 Page 分页的查询结果
         * 自动提取 response.total 或 response.count 并更新状态
         * 支持 total 和 count 字段，兼容 string 和 number 类型
         * @param queryResult - TanStack Query 的查询结果对象
         * @param currentPage - 当前页码
         * @param pageSize - 每页数量
         */
        handlePageQueryResult: useCallback((
            queryData?: {
                response?: {
                    total?: number | string | null;
                    count?: number | string | null;
                    [key: string]: unknown;
                };
            },
            currentPage?: number,
            pageSize?: number
        ) => {
            if (queryData?.response) {
                // 兼容 total 和 count 字段，支持 string 和 number 类型
                const rawTotal = queryData.response.total ?? queryData.response.count;
                const total = rawTotal !== null && rawTotal !== undefined ? Number(rawTotal) : null;

                // 如果返回了有效的 total（包括0），缓存并设置 count_num = false
                if (total !== null && !isNaN(total) && total >= 0) {
                    totalRef.current = total;
                    hasLoadedRef.current = true;
                    countNumRef.current = false;

                    // 如果提供了页码和页面大小，检查是否接近最后一页
                    if (currentPage !== undefined && pageSize !== undefined && pageSize > 0) {
                        const totalPages = Math.ceil(total / pageSize);
                        if (currentPage >= totalPages - 1) {
                            countNumRef.current = true;
                        }
                    }
                }
            }
        }, []),

        /**
         * 重置 count_num 为 true
         * 在筛选条件变化、页面大小变化等场景下调用
         */
        reset: useCallback(() => {
            countNumRef.current = true;
            hasLoadedRef.current = false;
            totalRef.current = null;
        }, []),

        /**
         * 手动设置 count_num 值（一般不需要使用）
         * @param value - 要设置的值
         */
        setCountNum: useCallback((value: boolean) => {
            countNumRef.current = value;
        }, []),

        /**
         * 手动设置 total 值（一般不需要使用）
         * @param value - 要设置的总数
         */
        setTotal: useCallback((value: number | null) => {
            totalRef.current = value;
        }, []),
    };
}
