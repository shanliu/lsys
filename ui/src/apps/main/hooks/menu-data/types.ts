import type { MenuData, MenuItem, MenuItemState } from '../../types/ui-menu'
import type { PermissionKey } from '../use-perm'

/**
 * 菜单配置项接口
 */
export interface MenuConfig extends MenuData {
  children?: MenuConfig[]
  permission?: string | PermissionKey
  /**
   * 权限校验失败时是否显示为禁用状态
   * - true: 显示为禁用菜单并提示错误信息
   * - false: 完全隐藏该菜单项（默认）
   */
  showDisabled?: boolean
  /**
   * 权限校验失败时的替代跳转路径
   * - 未授权访问该菜单时可跳转到该路径
   */
  noAuthPath?: string
}
//1 未授权隐藏
//2 未授权显示但禁用
//3 未授权显示并可跳转
/**
 * 处理后的菜单项（包含权限状态和错误信息）
 * 同时继承纯净的 MenuItem 和配置相关的 MenuConfig，扩展了权限相关功能
 */
export interface MenuItemWithPermission extends MenuItem, Omit<MenuConfig, 'id' | 'name' | 'path' | 'icon' | 'params' | 'children'> {
  params: Record<string, any> // MenuItemWithPermission 中 params 为必须字段，默认可为空对象
  state: MenuItemState
  children?: MenuItemWithPermission[]
}
