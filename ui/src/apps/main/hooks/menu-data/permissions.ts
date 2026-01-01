import { MenuItemState } from '@apps/main/types/ui-menu'
import type { PermissionKey, PermissionResult } from '../use-perm'
import { type MenuConfig, type MenuItemWithPermission } from './types'

/**
 * 提取菜单项的权限键值（只获取有权限key的菜单项）
 * 
 * 该函数遍历菜单树，提取所有需要权限检查的菜单项的权限键值，
 * 用于后续批量查询用户权限
 */
export function extractPermissionKeys(menuItems: MenuConfig[]): PermissionKey[] {
  const keys: PermissionKey[] = []

  function traverse(items: MenuConfig[]) {
    for (const item of items) {
      if (item.permission) {
        const key: PermissionKey = typeof item.permission === 'string'
          ? { name: item.permission }
          : { name: item.permission.name, data: item.permission.data }

        // Avoid duplicates by matching both name and data
        if (!keys.some(k => k.name === key.name && deepEqual(k.data, key.data))) {
          keys.push(key)
        }
      }
      if (item.children && item.children.length > 0) {
        traverse(item.children)
      }
    }
  }
  traverse(menuItems)
  return keys
}

function deepEqual(a: any, b: any): boolean {
  // Fast path - 严格相等
  if (a === b) return true

  // 非严格比较：将各种"假值"视为相等
  // 包括：null, undefined, false, 0, "0", ""
  function isFalsyValue(val: any): boolean {
    return val === null ||
      val === undefined ||
      val === false ||
      val === 0 ||
      val === "0" ||
      val === ""
  }

  // 如果两个值都是假值，认为相等
  if (isFalsyValue(a) && isFalsyValue(b)) {
    return true
  }

  // 如果类型不同，尝试转换后比较
  if (typeof a !== typeof b) {
    // 数字和字符串的宽松比较
    if ((typeof a === 'number' && typeof b === 'string') ||
      (typeof a === 'string' && typeof b === 'number')) {
      return String(a) === String(b)
    }
    // 布尔值和数字/字符串的宽松比较
    if (typeof a === 'boolean' || typeof b === 'boolean') {
      return Boolean(a) === Boolean(b)
    }
    return false
  }

  // 对象类型的深度比较
  if (typeof a === 'object' && a !== null && b !== null) {
    if (Array.isArray(a) !== Array.isArray(b)) return false

    if (Array.isArray(a)) {
      if (a.length !== b.length) return false
      return a.every((item, index) => deepEqual(item, b[index]))
    }

    const keysA = Object.keys(a)
    const keysB = Object.keys(b)
    if (keysA.length !== keysB.length) return false

    return keysA.every(key => keysB.includes(key) && deepEqual(a[key], b[key]))
  }

  // 字符串和数字的宽松比较
  if ((typeof a === 'string' || typeof a === 'number') &&
    (typeof b === 'string' || typeof b === 'number')) {
    return String(a) === String(b)
  }

  return false
}

/**
 * 根据权限结果过滤菜单项
 * 
 * 该函数会递归处理菜单项，根据用户权限决定菜单项的显示状态：
 * - 有权限：正常显示并启用
 * - 无权限但配置了 showDisabled：显示为禁用状态
 * - 无权限且未配置显示：完全隐藏
 * - 对于父级菜单：
 *   - 如果配置了权限且权限检查失败，按叶子节点逻辑处理（不显示子菜单，作为非组菜单样式）
 *   - 如果没有配置权限或权限检查通过，只有当至少有一个子菜单可见时才显示
 */
export function filterMenuByPermissions(
  menuItems: MenuConfig[],
  permissionResults: PermissionResult[]
): MenuItemWithPermission[] {
  // 创建权限结果映射表，便于快速查找
  const permissionMap: PermissionResult[] = permissionResults

  /**
   * 检查菜单项的权限
   * @param item 菜单配置项
   * @returns 权限结果对象或 null（无权限要求）
   */
  function checkPermission(item: MenuConfig): PermissionResult | null {
    if (!item.permission) {
      return null
    }

    const permissionKey: PermissionKey = typeof item.permission === 'string'
      ? { name: item.permission }
      : { name: item.permission.name, data: item.permission.data }

    // 找到匹配 name 和 data 的权限结果
    const permissionResult = permissionMap.find(pr => {
      const nameMatch = pr.name === permissionKey.name
      const dataMatch = deepEqual(pr['data'], permissionKey.data)
      return nameMatch && dataMatch
    })

    return permissionResult || null
  }

  /**
   * 处理单个菜单项
   * @param item 菜单配置项
   * @returns 处理后的菜单项或 null（表示隐藏）
   */
  function processMenuItem(item: MenuConfig): MenuItemWithPermission | null {
    // 如果有子项，先检查组菜单自身的权限
    if (item.children && item.children.length > 0) {
      const permissionResult = checkPermission(item)

      // 组菜单配置了权限
      if (permissionResult !== null) {
        // status = 1: 有权限，继续处理子菜单
        if (permissionResult.status === 1) {
          const filteredChildren = item.children
            .map(child => processMenuItem(child))
            .filter(Boolean) as MenuItemWithPermission[]

          // 如果有可见的子项，保留组菜单
          if (filteredChildren.length > 0) {
            return {
              ...item,
              path: item.path,
              params: item.params || {},
              children: filteredChildren,
              state: MenuItemState.ENABLED
            } as MenuItemWithPermission
          }
          return null // 没有可见子项，隐藏整个组菜单
        }

        // status = 2: 没有权限，按叶子节点逻辑处理（显示为非组菜单样式，不显示子菜单）
        if (permissionResult.status === 2) {
          // 如果配置了 showDisabled，显示为普通菜单（非组菜单样式）
          if (item.showDisabled) {
            return {
              ...item,
              path: item.noAuthPath ? item.noAuthPath : item.path,
              params: item.params || {},
              children: undefined, // 不显示子菜单，UI 会渲染为普通菜单样式
              state: item.noAuthPath ? MenuItemState.ENABLED : MenuItemState.DISABLED,
              tipsMessage: permissionResult.reason || '您没有访问此功能的权限'
            } as MenuItemWithPermission
          }
          // 未配置 showDisabled，完全隐藏
          return null
        }

        // status = 0: 失败不显示，直接隐藏
        return null
      }

      // 组菜单没有配置权限，正常处理子菜单
      const filteredChildren = item.children
        .map(child => processMenuItem(child))
        .filter(Boolean) as MenuItemWithPermission[]

      // 如果有可见的子项，保留组菜单
      if (filteredChildren.length > 0) {
        return {
          ...item,
          path: item.path,
          params: item.params || {},
          children: filteredChildren,
          state: MenuItemState.ENABLED
        } as MenuItemWithPermission
      }
      return null // 没有可见子项，隐藏整个组菜单
    }

    // 处理叶子节点
    const permissionResult = checkPermission(item)

    // 没有权限要求的菜单项，直接启用
    if (!permissionResult) {
      return {
        ...item,
        path: item.path,
        params: item.params || {},
        state: MenuItemState.ENABLED
      } as MenuItemWithPermission
    }

    // status = 1: 有权限，正常显示
    if (permissionResult.status === 1) {
      return {
        ...item,
        path: item.path,
        params: item.params || {},
        state: MenuItemState.ENABLED
      } as MenuItemWithPermission
    }

    // status = 2: 没有权限但要显示错误提示（根据 showDisabled 配置决定）
    if (permissionResult.status === 2) {
      if (item.showDisabled) {
        return {
          ...item,
          path: item.noAuthPath ? item.noAuthPath : item.path,
          params: item.params || {},
          state: item.noAuthPath ? MenuItemState.ENABLED : MenuItemState.DISABLED,
          tipsMessage: permissionResult.reason || '您没有访问此功能的权限'
        } as MenuItemWithPermission
      }
      return null // 隐藏
    }

    // status = 0: 失败不显示，直接隐藏
    return null
  }

  return menuItems
    .map(item => processMenuItem(item))
    .filter(Boolean) as MenuItemWithPermission[]
}
