import { userQueryKey } from '@apps/main/lib/auth-utils'
import { checkPerm } from '@shared/apis/auth/perm'
import { useQuery } from '@tanstack/react-query'
import { useMemo } from 'react'

export interface PermissionKey {
  name: string
  data?: any
}


// 权限检查结果
// status: 0 = 失败不显示, 1 = 成功, 2 = 失败提示错误
export interface PermissionResult {
  name: string
  data?: any
  status: number
  reason?: string
}

/**
 * 权限管理 Hook
 * 使用 TanStack Query 进行权限检查和缓存
 */
export function usePerm(permission: (string | PermissionKey)[]): {
  isLoading: boolean
  isError: boolean
  error: any
  result: PermissionResult[]
} {
  // 统一转换为 PermissionKey[]
  const permissionKeys: PermissionKey[] = useMemo(() =>
    permission.map(p => typeof p === 'string' ? { name: p } : p)
    , [permission])

  // 创建稳定的查询键

  const results = useQuery({
    queryKey: userQueryKey('permission', permissionKeys),
    queryFn: ({ signal }) => checkPerm(permissionKeys, { signal }),
    enabled: permissionKeys.length > 0,
    staleTime: 2 * 60 * 1000, // 权限缓存2分钟
    retry: 2, // 权限检查失败时重试2次
  })

  const result: PermissionResult[] = useMemo(() => {
    const list = results.data?.response ?? []

    // 如果API返回错误状态，生成默认的权限拒绝结果
    if (results.data?.status === false && permissionKeys.length > 0) {
      return permissionKeys.map(key => ({
        name: key.name,
        status: 0 as const, // 失败不显示
        reason: `权限检查失败: ${results.data?.message || '未知错误'}`
      }))
    }

    // 否则使用原来的转换逻辑
    return Array.isArray(list)
      ? list.map((item: any) => ({
        name: item.name,
        data: item.data,
        status: item.status as 0 | 1 | 2, // 0=失败不显示, 1=成功, 2=失败提示错误
        reason: item.msg,
      }))
      : []
  }, [results.data, permissionKeys])

  return {
    isLoading: results.isLoading,
    isError: results.isError,
    error: results.error,
    result
  }
}
