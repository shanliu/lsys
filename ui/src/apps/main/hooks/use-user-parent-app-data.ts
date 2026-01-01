import { appParentList, type AppParentListParamType } from '@shared/apis/user/app'
import { useInfiniteQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useMemo } from 'react'

/**
 * 父应用信息类型
 */
export interface UserParentAppInfo {
  id: string
  name: string
  client_id: string
  user_id: string
  change_time: number
  user_account: string
  user_data: string
  user_nickname: string
}

/**
 * 父应用数据类型定义
 */
export interface UserParentAppData {
  apps: UserParentAppInfo[]
  isLoading: boolean
  isError: boolean
  error: string | null
  hasMore: boolean
  loadMore: () => Promise<void>
  refresh: () => Promise<void>
  isLoadingMore: boolean
  search: (keyword?: string) => Promise<void>
  isSearching: boolean
}

/**
 * useParentAppData Hook 配置选项
 */
export interface UseUserParentAppDataOptions {
  /** 搜索关键词 */
  keyword?: string
  /** 每页加载数量，默认为20 */
  pageSize?: number
  /** 是否自动加载，默认为true */
  autoLoad?: boolean
}

/**
 * 获取父应用数据的 Hook
 * 使用 TanStack Query 来管理数据获取和缓存
 * 支持分页加载和关键词搜索
 */
export function useUserParentAppData(options: UseUserParentAppDataOptions = {}): UserParentAppData {
  const { keyword, pageSize = 3, autoLoad = true } = options
  const queryClient = useQueryClient()

  // 转换API数据为ParentAppInfo格式
  const transformAppData = useCallback((rawApps: any[]): UserParentAppInfo[] => {
    return rawApps.map(rawApp => ({
      id: rawApp.id,
      name: rawApp.name,
      client_id: rawApp.client_id,
      user_id: rawApp.user_id,
      change_time: rawApp.change_time,
      user_account: rawApp.user_data.user_account,
      user_data: rawApp.user_data.user_data,
      user_nickname: rawApp.user_data.user_nickname
    }))
  }, [])

  // 查询键工厂
  const queryKeyFactory = useMemo(() => ({
    all: ['parentApps'] as const,
    lists: () => [...queryKeyFactory.all, 'list'] as const,
    list: (params: { keyword?: string; pageSize: number }) => [...queryKeyFactory.lists(), params] as const,
  }), [])

  // 无限查询父应用列表
  const parentAppsQuery = useInfiniteQuery({
    queryKey: queryKeyFactory.list({ keyword, pageSize }),
    queryFn: async ({ pageParam = 1, signal }) => {
      const param: AppParentListParamType = {
        page: { page: pageParam, limit: pageSize },
        count_num: true,
        ...(keyword && { key_word: keyword })
      }

      const result = await appParentList(param, { signal })

      if (result.status && result.response) {
        const rawApps = result.response.data || []
        const apps = transformAppData(rawApps)
        const total = result.response.total || 0
        const currentTotal = (pageParam - 1) * pageSize + apps.length
        const hasMore = currentTotal < total

        return {
          apps,
          total,
          currentTotal,
          nextPage: hasMore ? pageParam + 1 : undefined,
        }
      }

      return {
        apps: [],
        total: 0,
        currentTotal: 0,
        nextPage: undefined,
      }
    },
    initialPageParam: 1,
    getNextPageParam: (lastPage) => lastPage.nextPage,
    enabled: autoLoad,
    staleTime: 5 * 60 * 1000, // 5分钟
  })

  // 合并所有分页数据
  const apps = useMemo(() => {
    const pagesData = parentAppsQuery.data?.pages || []
    return pagesData.flatMap(page => page.apps)
  }, [parentAppsQuery.data])

  // 加载更多
  const loadMore = useCallback(async () => {
    if (parentAppsQuery.hasNextPage && !parentAppsQuery.isFetchingNextPage) {
      await parentAppsQuery.fetchNextPage()
    }
  }, [parentAppsQuery])

  // 搜索功能
  const search = useCallback(async (searchKeyword?: string) => {
    const newKeyword = searchKeyword !== undefined ? searchKeyword : keyword

    // 刷新查询以使用新的搜索关键词
    await queryClient.invalidateQueries({
      queryKey: queryKeyFactory.list({ keyword: newKeyword, pageSize })
    })
  }, [keyword, pageSize, queryClient, queryKeyFactory])

  // 刷新数据
  const refresh = useCallback(async () => {
    await queryClient.invalidateQueries({
      queryKey: queryKeyFactory.list({ keyword, pageSize })
    })
  }, [keyword, pageSize, queryClient, queryKeyFactory])

  return {
    apps,
    isLoading: parentAppsQuery.isLoading,
    isError: parentAppsQuery.isError,
    error: parentAppsQuery.error?.message || null,
    hasMore: parentAppsQuery.hasNextPage || false,
    isLoadingMore: parentAppsQuery.isFetchingNextPage,
    isSearching: parentAppsQuery.isFetching && !parentAppsQuery.isLoading,
    loadMore,
    refresh,
    search
  }
}
