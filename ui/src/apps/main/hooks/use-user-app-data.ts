import { appQueryKey } from '@apps/main/lib/auth-utils'
import { appList, type AppListParamType } from '@shared/apis/user/app'
import { useInfiniteQuery, useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useMemo, useRef, useState } from 'react'

/**
 * 应用信息类型 - 卡片式应用简化版
 */
export interface UserAppInfo {
  id: number
  name: string
  client_id: string
  status: number
  user_id: number
  user_account: string
  user_data: string
  user_nickname: string
}

/**
 * 应用数据类型定义
 */
export interface UserAppData {
  apps: UserAppInfo[]
  isLoading: boolean
  isError: boolean
  error: string | null
  hasMore: boolean
  loadMore: () => Promise<void>
  refresh: () => Promise<void>
  isLoadingMore: boolean
  /** 是否已经加载了列表（懒加载模式下使用） */
  isListLoaded: boolean
  /** 搜索功能 */
  search: (keyword?: string) => Promise<void>
  /** 是否正在搜索 */
  isSearching: boolean
}

/**
 * useAppData Hook 配置选项
 */
export interface UseUserAppDataOptions {
  /** 优先加载的应用ID，会在第一页优先显示 */
  priorityAppId?: number
  /** 每页加载数量，默认为20 */
  pageSize?: number
  /** 是否自动加载，默认为true */
  autoLoad?: boolean
  /** 应用状态过滤：0-待审核，1-正常，2-已禁用 */
  status?: number
  /** 客户端ID过滤 */
  client_id?: string
}

/**
 * 获取应用数据的 Hook
 * 使用 TanStack Query 来管理数据获取和缓存
 * 支持分页加载和优先加载指定应用
 */
export function useUserAppData(options: UseUserAppDataOptions = {}): UserAppData {
  const { priorityAppId, pageSize = 10, autoLoad = true, status, client_id } = options
  const queryClient = useQueryClient()

  // 懒加载状态控制
  const [isManuallyTriggered, setIsManuallyTriggered] = useState(false)
  const shouldLoadRef = useRef(autoLoad)

  // 搜索状态控制
  const [isSearching, setIsSearching] = useState(false)

  // 转换API数据为AppInfo格式
  const transformAppData = useCallback((rawApps: any[]): UserAppInfo[] => {
    return rawApps.map(rawApp => ({
      id: rawApp.id,
      name: rawApp.name,
      client_id: rawApp.client_id,
      status: rawApp.status,
      user_id: rawApp.user_data?.id || 0,
      user_account: rawApp.user_data?.user_account || '',
      user_data: rawApp.user_data?.user_data || '',
      user_nickname: rawApp.user_data?.user_nickname || ''
    }))
  }, [])

  // 查询键工厂 - 包含 status 和 client_id 参数以确保缓存隔离
  const queryKeyFactory = useMemo(() => ({
    all: ['apps'] as const,
    lists: () => [...queryKeyFactory.all, 'list'] as const,
    list: (pageSize: number, status?: number, client_id?: string) => [...queryKeyFactory.lists(), { pageSize, status, client_id }] as const,
  }), [])

  // 优先应用单独查询 - 构建 queryKey，确保与 notify-page 等其他查询一致
  const hasFilters = status !== undefined || client_id
  const priorityAppQueryKey = useMemo(() => {
    if (!priorityAppId) {
      return ['apps', 'priority', 'disabled']
    }
    // 当没有过滤条件时，使用简单的 appQueryKey，以便与其他查询共用缓存
    if (!hasFilters) {
      return appQueryKey(priorityAppId)
    }
    // 有过滤条件时，添加参数对象以区分不同的查询结果
    return [...appQueryKey(priorityAppId), { status, client_id }]
  }, [priorityAppId, hasFilters, status, client_id])

  const priorityAppQuery = useQuery({
    queryKey: priorityAppQueryKey,
    queryFn: async ({ signal }) => {
      if (!priorityAppId) return null

      const param: AppListParamType = {
        page: { page: 1, limit: 1 },
        count_num: false,
        app_id: priorityAppId,
        ...(status !== undefined && { status }),
        ...(client_id && { client_id })
      }

      const result = await appList(param, { signal })

      if (result.status && result.response) {
        const rawApps = result.response.data || []
        if (rawApps.length > 0) {
          return rawApps[0]
        }
      }

      return null
    },
    enabled: !!priorityAppId && (autoLoad || isManuallyTriggered),
    staleTime: 5 * 60 * 1000,
  })

  // 转换优先应用数据
  const priorityApp = useMemo(() => {
    if (priorityAppQuery.data) {
      return transformAppData([priorityAppQuery.data])[0]
    }
    return null
  }, [priorityAppQuery.data, transformAppData])

  // 判断是否应该自动加载列表 - 改为支持懒加载
  // 当有搜索条件(client_id)时，也应该自动加载以显示搜索结果
  const shouldAutoLoadList = autoLoad || isManuallyTriggered || !!client_id

  // 主应用列表无限查询
  const appsQuery = useInfiniteQuery({
    queryKey: queryKeyFactory.list(pageSize, status, client_id),
    queryFn: async ({ pageParam = 1, signal }) => {
      const param: AppListParamType = {
        page: { page: pageParam, limit: pageSize },
        count_num: true,
        ...(status !== undefined && { status }),
        ...(client_id && { client_id })
      }

      const result = await appList(param, { signal })

      if (result.status && result.response) {
        const rawApps = result.response.data || []
        const apps = transformAppData(rawApps)
        const total = result.response.total || 0
        const hasMore = pageParam * pageSize < total

        return {
          apps,
          nextPage: hasMore ? pageParam + 1 : undefined,
          total
        }
      }

      return {
        apps: [],
        nextPage: undefined,
        total: 0
      }
    },
    initialPageParam: 1,
    getNextPageParam: (lastPage) => lastPage.nextPage,
    enabled: shouldAutoLoadList,
    staleTime: 5 * 60 * 1000,
  })

  // 获取所有已加载的应用数据
  const allPageApps = useMemo(() => {
    const pagesData = appsQuery.data?.pages || []
    return pagesData.flatMap(page => page.apps)
  }, [appsQuery.data])

  // 最终合并数据 - 智能去重
  const combinedApps = useMemo(() => {
    // 如果没有优先应用数据，直接返回列表数据
    if (!priorityApp) {
      return allPageApps
    }

    // 检查优先应用是否已经在列表中
    const existsInList = allPageApps.find(app => app.id === priorityApp.id)

    if (existsInList) {
      // 如果已存在，移动到最前面（保证优先显示）
      const filteredApps = allPageApps.filter(app => app.id !== priorityApp.id)
      return [priorityApp, ...filteredApps]
    } else {
      // 如果不存在，直接插入到最前面
      return [priorityApp, ...allPageApps]
    }
  }, [priorityApp, allPageApps])

  // 计算状态 - 简化加载状态逻辑
  const isLoading = appsQuery.isLoading || (!!priorityAppId && priorityAppQuery.isLoading)
  const isError = appsQuery.isError || priorityAppQuery.isError
  const error = appsQuery.error?.message || priorityAppQuery.error?.message || null

  // 加载更多 - 统一处理初始加载和分页加载，支持懒加载触发
  const loadMore = useCallback(async () => {
    // 如果是懒加载模式且尚未手动触发，先触发加载
    if (!autoLoad && !isManuallyTriggered) {
      setIsManuallyTriggered(true)
      shouldLoadRef.current = true
      return
    }

    // 如果列表查询未启用或无数据，先启用查询
    if (!appsQuery.dataUpdatedAt) {
      if (!autoLoad && !isManuallyTriggered) {
        setIsManuallyTriggered(true)
        shouldLoadRef.current = true
      }
      await appsQuery.refetch()
    } else if (appsQuery.hasNextPage && !appsQuery.isFetchingNextPage) {
      await appsQuery.fetchNextPage()
    }
  }, [appsQuery, autoLoad, isManuallyTriggered])

  // 搜索功能
  const search = useCallback(async (keyword?: string) => {
    setIsSearching(true)
    try {
      if (!keyword?.trim()) {
        // 如果没有搜索关键词，刷新原始列表
        await queryClient.invalidateQueries({
          queryKey: queryKeyFactory.list(pageSize, status, client_id)
        })
      } else {
        // 使用 client_id 进行搜索
        await queryClient.invalidateQueries({
          queryKey: queryKeyFactory.list(pageSize, status, keyword)
        })
      }
    } finally {
      setIsSearching(false)
    }
  }, [pageSize, status, client_id, queryClient, queryKeyFactory])

  // 刷新数据
  const refresh = useCallback(async () => {
    const promises = []

    promises.push(queryClient.invalidateQueries({
      queryKey: queryKeyFactory.list(pageSize, status, client_id)
    }))

    if (priorityAppId) {
      promises.push(queryClient.invalidateQueries({
        queryKey: appQueryKey(priorityAppId)
      }))
    }

    await Promise.all(promises)
  }, [priorityAppId, pageSize, status, client_id, queryClient, queryKeyFactory])

  return {
    apps: combinedApps,
    isLoading,
    isError,
    error,
    hasMore: appsQuery.hasNextPage || false,
    isLoadingMore: appsQuery.isFetchingNextPage,
    loadMore,
    refresh,
    isListLoaded: !!appsQuery.data,
    search,
    isSearching,
  }
}
