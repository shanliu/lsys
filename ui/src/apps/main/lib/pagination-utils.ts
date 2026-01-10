import type { LimitType } from "@shared/types/base-schema";
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
 * 计算下一页的 pos 和 eq_pos 参数
 * @param options 配置选项
 * @param options.maxId 当前页面数据中的最大ID（最后一条记录的ID）
 * @param options.next 接口返回的 next 字段值
 * @param options.currentForward 当前的 forward 状态
 * @returns 包含 pos 和 eq_pos 的对象
 */
function OffsetPaginationCalcNext(options: {
    maxId: number | null;
    next: number | null | undefined;
    currentForward: boolean;
}): { pos: number | null; eq_pos: boolean } {
    const { maxId, next, currentForward } = options;

    let pos: number | null = null;
    let eq_pos = false;

    // 获取当前页面最后一条记录的ID作为默认pos
    if (maxId !== null) {
        pos = maxId;
    }

    // 如果接口返回了next且当前不是forward模式，使用API返回的next值
    if (next && !currentForward) {
        pos = next;
        eq_pos = true;
    }

    return { pos, eq_pos };
}

/**
 * 计算上一页的 pos 和 eq_pos 参数
 * @param options 配置选项
 * @param options.minId 当前页面数据中的最小ID（第一条记录的ID）
 * @param options.next 接口返回的 next 字段值
 * @param options.currentForward 当前的 forward 状态
 * @returns 包含 pos 和 eq_pos 的对象
 */
function OffsetPaginationCalcPrev(options: {
    minId: number | null;
    next: number | null | undefined;
    currentForward: boolean;
}): { pos: number | null; eq_pos: boolean } {
    const { minId, next, currentForward } = options;

    let pos: number | null = null;
    let eq_pos = false;

    // 获取当前页面第一条记录的ID作为默认pos
    if (minId !== null) {
        pos = minId;
    }

    // 如果当前是forward模式且接口返回了next，使用API返回的next值
    if (currentForward && next) {
        pos = next;
        eq_pos = true;
    }

    return { pos, eq_pos };
}

/**
 * 判断是否可以进入下一页
 * @param options 配置选项
 * @param options.currentForward 当前的 forward 状态
 * @param options.next 接口返回的 next 字段值
 * @param options.currentPos 当前的 pos 值
 * @returns 是否可以进入下一页
 */
function OffsetPaginationCanNext(options: {
    currentForward: boolean;
    next: number | null | undefined;
    currentPos: number | null;
}): boolean {
    const { currentForward, next, currentPos } = options;

    // 下一页：在forward=false模式下有next可以继续向后翻，或在forward=true模式下可以返回到更旧数据
    return (
        (!currentForward && next !== null && next !== undefined) ||
        (currentForward && currentPos !== null)
    );
}

/**
 * 判断是否可以进入上一页
 * @param options 配置选项
 * @param options.currentForward 当前的 forward 状态
 * @param options.next 接口返回的 next 字段值
 * @param options.currentPos 当前的 pos 值
 * @returns 是否可以进入上一页
 */
function OffsetPaginationCanPrev(options: {
    currentForward: boolean;
    next: number | null | undefined;
    currentPos: number | null;
}): boolean {
    const { currentForward, next, currentPos } = options;

    // 上一页：在forward=true模式下有next可以继续向前翻，或在forward=false模式下不在首页可以切换到向前模式
    return (
        (currentForward && next !== null && next !== undefined) ||
        (!currentForward && currentPos !== null)
    );
}

/**
 * 创建偏移分页处理器
 * 用于统一管理分页相关的逻辑，避免在每个页面重复定义
 *
 * @example
 * ```tsx
 * const { handleNextPage, handlePrevPage, canGoNext, canGoPrev } = useOffsetPaginationHandlers({
 *   minId: messages.length > 0 ? messages[0].id : null,
 *   maxId: messages.length > 0 ? messages[messages.length - 1].id : null,
 *   pagination: pagination, // 直接传入整个 LimitType 对象
 *   nextPageStartPos: nextPageStartPos,
 *   searchGo: (param) => navigate({ search: { ...filterParam, ...param } }),
 *   defaultForward: false, // 默认排序方向（从大到小为 false，从小到大为 true）
 *   setPagination, // 可选
 * })
 * ```
 */
export function useOffsetPaginationHandlers(options: {
    /** 当前页面数据中的最小ID（第一条记录的ID） */
    minId: number | null;
    /** 当前页面数据中的最大ID（最后一条记录的ID） */
    maxId: number | null;
    /** 当前分页状态 - 使用全局统一的 LimitType */
    pagination: LimitType;
    /** 下一页起始位置（API返回的next字段） */
    nextPageStartPos: number | null | undefined;
    /** 搜索导航函数 - 用于更新URL参数 */
    searchGo: (
        param: { pos: number | null; forward: boolean; eq_pos: boolean } & Record<
            string,
            any
        >,
    ) => void;
    /** 默认排序方向 - false: 从大到小(新到旧), true: 从小到大(旧到新)。默认为 false */
    defaultForward?: boolean;
    /** 可选：本地分页状态更新函数 */
    setPagination?: React.Dispatch<React.SetStateAction<any>>;
}): {
    /** 下一页处理函数 */
    handleNextPage: () => void;
    /** 上一页处理函数 */
    handlePrevPage: () => void;
    /** 是否可以进入下一页 */
    canGoNext: boolean;
    /** 是否可以进入上一页 */
    canGoPrev: boolean;
} {
    const {
        minId,
        maxId,
        pagination,
        nextPageStartPos,
        searchGo,
        defaultForward = false,
        setPagination,
    } = options;

    // 下一页处理函数
    const handleNextPage = () => {
        const { pos, eq_pos } = OffsetPaginationCalcNext({
            maxId,
            next: nextPageStartPos,
            currentForward: pagination.forward || defaultForward,
        });

        // 更新 URL（分页状态由 URL 参数驱动）
        searchGo({
            pos,
            forward: defaultForward, // 使用配置的默认排序方向
            eq_pos, // 包含pos位置的数据
        });

        // 可选：同步更新本地状态
        if (setPagination) {
            setPagination((prev: any) => ({
                ...prev,
                pos,
                forward: defaultForward,
                eq_pos,
            }));
        }
    };

    // 上一页处理函数
    const handlePrevPage = () => {
        const { pos, eq_pos } = OffsetPaginationCalcPrev({
            minId,
            next: nextPageStartPos,
            currentForward: pagination.forward || defaultForward,
        });

        // 更新 URL（分页状态由 URL 参数驱动）
        searchGo({
            pos,
            forward: !defaultForward, // 反向查询
            eq_pos, // 不包含pos位置的数据
        });

        // 可选：同步更新本地状态
        if (setPagination) {
            setPagination((prev: any) => ({
                ...prev,
                pos,
                forward: !defaultForward,
                eq_pos,
            }));
        }
    };

    // 分页按钮状态判断
    const canGoNext = OffsetPaginationCanNext({
        currentForward: pagination.forward || defaultForward,
        next: nextPageStartPos,
        currentPos: pagination.pos ?? null,
    });

    const canGoPrev = OffsetPaginationCanPrev({
        currentForward: pagination.forward || defaultForward,
        next: nextPageStartPos,
        currentPos: pagination.pos ?? null,
    });

    return {
        handleNextPage,
        handlePrevPage,
        canGoNext,
        canGoPrev,
    };
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
         * 自动提取 response.total 和 response.next 并更新状态
         * @param queryResult - TanStack Query 的查询结果对象
         */
        handleLimitQueryResult: useCallback((queryData: {
            response?: {
                total?: number | null;
                next?: number | null;
            };
        }) => {
            if (queryData?.response) {
                const { total, next } = queryData.response;

                // 如果返回了有效的 total（包括0），缓存并设置 count_num = false
                if (total !== null && total !== undefined && total >= 0) {
                    totalRef.current = total;
                    hasLoadedRef.current = true;
                    countNumRef.current = false;
                }

                // 如果没有下一页，重置 count_num = true
                if (hasLoadedRef.current && (next === null || next === undefined)) {
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
