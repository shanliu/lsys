import type { LucideIcon } from 'lucide-react'



/**
 * 纯粹的菜单项定义
 * 用于通用菜单组件，不包含权限和状态相关信息
 */
export interface MenuData {
  name: string
  id?: string
  path?: string
  icon?: LucideIcon
  params?: Record<string, any>
}

/**
 * 菜单项状态枚举
 */
export enum MenuItemState {
  ENABLED,
  DISABLED
}


/**
 * 纯粹的菜单项定义
 * 用于通用菜单组件，不包含权限和状态相关信息
 */
export interface MenuItem extends MenuData {
  children?: MenuItem[]
  state?: MenuItemState
  tipsMessage?: string
  isActive?: boolean
}


