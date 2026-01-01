import { appList, type AppListParamType } from '@shared/apis/admin/app'
import { useInfiniteQuery, useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useMemo, useState } from 'react'

/**
 * 系统应用信息类型 - 管理端应用简化版
 */
export interface SystemAppInfo {
  id: number
  name: string
  client_id: string | null
  status: number
  user_id: number
  user_account: string
  user_data: string
  user_nickname: string
}

/**
 * 系统应用数据类型定义
 */
export interface SystemAppData {
  apps: SystemAppInfo[]
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
 * useSystemAppData Hook 配置选项
 */
export interface UseSystemAppDataOptions {
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
  /** 用户ID过滤 */
  user_id?: number
  /** 应用名称过滤 */
  app_name?: string
}

/**
 * 获取系统应用数据的 Hook
 * 使用 TanStack Query 来管理数据获取和缓存
 * 支持分页加载和优先加载指定应用
 */
export function useSystemAppData(options: UseSystemAppDataOptions = {}): SystemAppData {
  const {
    priorityAppId,
    pageSize = 20,
    autoLoad = true,
    status,
    client_id,
    user_id,
    app_name
  } = options
  const queryClient = useQueryClient()

  // 懒加载状态控制
  const [isManuallyTriggered, setIsManuallyTriggered] = useState(false)

  // 搜索状态控制
  const [isSearching, setIsSearching] = useState(false)

  // 转换API数据为SystemAppInfo格式
  const transformAppData = useCallback((rawApps: any[]): SystemAppInfo[] => {
    return rawApps.map(rawApp => ({
      id: rawApp.id,
      name: rawApp.name,
      client_id: rawApp.client_id || null,
      status: rawApp.status,
      user_id: rawApp.user_data?.id || 0,
      user_account: rawApp.user_data?.user_account || '',
      user_data: rawApp.user_data?.user_data || '',
      user_nickname: rawApp.user_data?.user_nickname || ''
    }))
  }, [])

  // 查询键工厂 - 包含所有参数以确保缓存隔离
  const queryKeyFactory = useMemo(() => {
    const base = ['systemApps'] as const
    return {
      all: base,
      lists: () => [...base, 'list'] as const,
      list: (pageSize: number, status?: number, client_id?: string, user_id?: number, app_name?: string) =>
        [...base, 'list', { pageSize, status, client_id, user_id, app_name }] as const,
      priority: (appId: number) => [...base, 'priority', appId] as const,
    }
  }, [])

  // 优先应用单独查询
  const priorityAppQuery = useQuery({
    queryKey: queryKeyFactory.priority(priorityAppId || 0),
    queryFn: async ({ signal }) => {
      if (!priorityAppId) return null

      const param: AppListParamType = {
        page: { page: 1, limit: 1 },
        count_num: false,
        app_id: priorityAppId,
        detail_data: false,
        ...(status !== undefined && { status }),
        ...(client_id && { client_id }),
        ...(user_id && { user_id }),
        ...(app_name && { app_name })
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
  const shouldAutoLoadList = autoLoad || isManuallyTriggered

  // 主应用列表无限查询
  const appsQuery = useInfiniteQuery({
    queryKey: queryKeyFactory.list(pageSize, status, client_id, user_id, app_name),
    queryFn: async ({ pageParam = 1, signal }) => {
      const param: AppListParamType = {
        page: { page: pageParam, limit: pageSize },
        count_num: true,
        detail_data: false,
        ...(status !== undefined && { status }),
        ...(client_id && { client_id }),
        ...(user_id && { user_id }),
        ...(app_name && { app_name })
      }

      const result = await appList(param, { signal })

      if (result.status && result.response) {
        const apps = transformAppData(result.response.data)
        const total = result.response.total || 0
        const hasNextPage = total > pageParam * pageSize

        return {
          apps,
          nextPage: hasNextPage ? pageParam + 1 : undefined,
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
    if (!priorityApp) {
      return allPageApps
    }

    // 检查优先应用是否已存在于列表中
    const priorityExists = allPageApps.some(app => app.id === priorityApp.id)

    if (priorityExists) {
      // 将优先应用移到前面
      const filteredApps = allPageApps.filter(app => app.id !== priorityApp.id)
      return [priorityApp, ...filteredApps]
    } else {
      // 优先应用不在列表中，直接添加到前面
      return [priorityApp, ...allPageApps]
    }
  }, [priorityApp, allPageApps])

  // 计算状态 - 简化加载状态逻辑
  const isLoading = appsQuery.isLoading || (!!priorityAppId && priorityAppQuery.isLoading)
  const isError = appsQuery.isError || priorityAppQuery.isError
  const error = appsQuery.error?.message || priorityAppQuery.error?.message || null

  // 加载更多 - 统一处理初始加载和分页加载，支持懒加载触发
  const loadMore = useCallback(async () => {
    if (!autoLoad && !isManuallyTriggered) {
      setIsManuallyTriggered(true)
      return
    }

    if (appsQuery.hasNextPage && !appsQuery.isFetchingNextPage) {
      await appsQuery.fetchNextPage()
    }
  }, [appsQuery, autoLoad, isManuallyTriggered])

  // 搜索功能
  const search = useCallback(async (keyword?: string) => {
    setIsSearching(true)
    try {
      await queryClient.invalidateQueries({
        queryKey: queryKeyFactory.list(pageSize, status, client_id, user_id, keyword)
      })
    } finally {
      setIsSearching(false)
    }
  }, [pageSize, status, client_id, user_id, queryClient, queryKeyFactory])

  // 刷新数据
  const refresh = useCallback(async () => {
    await Promise.all([
      priorityAppId && queryClient.invalidateQueries({
        queryKey: queryKeyFactory.priority(priorityAppId)
      }),
      queryClient.invalidateQueries({
        queryKey: queryKeyFactory.list(pageSize, status, client_id, user_id, app_name)
      })
    ])
  }, [priorityAppId, pageSize, status, client_id, user_id, app_name, queryClient, queryKeyFactory])

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
