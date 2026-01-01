import { MenuItemState } from '@apps/main/types/ui-menu'
import { useMemo } from 'react'
import {
  extractPermissionKeys,
  filterMenuByPermissions,
  getMenuData,
  MenuItemWithPermission,
  type MenuParamsMap,
} from './menu-data'
import { usePerm } from './use-perm'



interface NavigationMenuHookResult {
  isLoading: boolean
  isError: boolean
  error: string | null
  result: MenuItemWithPermission[]
}

/**
 * 菜单数据 Hook - 支持权限过滤，带类型约束
 * @param menuType - 菜单类型：'system' | 'user' | 'app' 
 * @param params - 菜单参数对象，类型根据 menuType 自动推断
 */
export function useMenu<T extends keyof MenuParamsMap>(
  menuType: T,
  ...args: MenuParamsMap[T] extends undefined ? [] : [MenuParamsMap[T]]
): NavigationMenuHookResult {

  const menuData = useMemo(() => getMenuData(menuType, ...args), [menuType, args])

  // 提取需要检查的权限键值
  const permissionKeys = useMemo(() => {
    return extractPermissionKeys(menuData)
  }, [menuData])

  // 获取权限检查结果
  const { result: permissionResults, isLoading, isError, error } = usePerm(permissionKeys)

  // 根据权限结果过滤菜单
  const result = useMemo(() => {
    if (!permissionResults) {
      return menuData.map(item => ({
        ...item,
        state: MenuItemState.ENABLED,
        children: item.children?.map(child => ({
          ...child,
          state: MenuItemState.ENABLED
        }))
      } as MenuItemWithPermission))
    }

    return filterMenuByPermissions(menuData, permissionResults)
  }, [menuData, permissionResults])

  return {
    isLoading,
    isError,
    error: error,
    result
  }
}
