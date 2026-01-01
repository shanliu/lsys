import { getSystemMenu } from './data/system-menus'
import { getAppMenu } from './data/user-app-menus'
import { getUserMenu } from './data/user-menus'
import { type MenuConfig } from './types'

// 重新导出所有相关的函数和类型
export { extractPermissionKeys, filterMenuByPermissions } from './permissions'
export { type MenuConfig, type MenuItemWithPermission } from './types'

/**
 * 为菜单项自动生成ID
 * @param menuItems 菜单配置数组
 * @param prefix ID前缀，默认为 'menu'
 * @param parentIndex 父级索引，用于生成层级ID
 * @returns 带有自动生成ID的菜单配置
 */
function assignMenuIds(
  menuItems: MenuConfig[],
  prefix: string = 'menu',
  parentIndex?: string
): MenuConfig[] {
  return menuItems.map((item, index) => {
    // 生成当前项的ID
    const currentId = parentIndex
      ? `${prefix}-${parentIndex}-${index}`
      : `${prefix}-${index}`

    // 如果已有ID，保留原ID；否则使用生成的ID
    const id = item.id || currentId

    // 处理子菜单
    const children = item.children
      ? assignMenuIds(item.children, prefix, parentIndex ? `${parentIndex}-${index}` : String(index))
      : undefined

    return {
      ...item,
      id,
      children
    }
  })
}

/**
 * 菜单数据映射 - 完全类型化的函数映射
 */
export const menuGetters = {
  system: getSystemMenu,
  user: getUserMenu,
  user_app: getAppMenu
} as const

/**
 * 从 menuGetters 推断参数类型映射
 */
export type MenuParamsMap = {
  [K in keyof typeof menuGetters]: typeof menuGetters[K] extends (...args: infer P) => any
  ? P extends []
  ? undefined
  : P extends [infer First]
  ? First
  : P[0]
  : never
}

/**
 * 获取指定类型的菜单数据 - 完全自动化，类型安全
 * 自动为菜单项生成ID
 */
export function getMenuData<T extends keyof MenuParamsMap>(
  menuType: T,
  ...args: MenuParamsMap[T] extends undefined ? [] : [MenuParamsMap[T]]
): MenuConfig[] {
  const menuGetter = menuGetters[menuType]
  const rawMenuData = (menuGetter as any)(...args)

  // 自动为菜单项生成ID
  return assignMenuIds(rawMenuData, menuType)
}

export { menuGetters as menus }
