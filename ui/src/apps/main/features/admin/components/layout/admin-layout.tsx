import { SidebarMenu, SidebarMenuError, SidebarMenuSkeleton } from '@apps/main/components/local/sidebar-menu'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { PageSkeletonTable } from '@shared/components/custom/page-placeholder/skeleton-table'
import { Sidebar, SidebarContent, SidebarProvider, SidebarRail } from '@shared/components/ui/sidebar'
import { useMenu } from '@apps/main/hooks/use-menu-data'
import { cn } from '@shared/lib/utils'
import { MenuItem } from '@apps/main/types/ui-menu'
import { Outlet, useLocation, useMatches } from '@tanstack/react-router'
import { useCallback, useMemo } from 'react'

interface AdminLayoutProps {
  className?: string
}

// 管理员布局的路径匹配函数
function isPathActive(
  menuPath: string | undefined,
  currentPathname: string,
  currentSearch: string,
  matches: any[]
): boolean {
  if (!menuPath) return false

  // 分离路径和查询参数
  const [menuPathOnly, menuQuery] = menuPath.split('?')
  const currentFullPath = currentPathname + (currentSearch ? currentSearch : '')

  // 1. 如果菜单路径包含查询参数，需要完整匹配路径和查询参数
  if (menuQuery) {
    // 构建完整的菜单路径(包含查询参数)
    const fullMenuPath = menuPathOnly + '?' + menuQuery
    // 完整匹配当前路径和查询参数
    if (currentFullPath === fullMenuPath) {
      return true
    }
    
    // 对于相同基础路径但不同查询参数的情况（如标签页切换），也认为是激活状态
    // 只要基础路径匹配即可
    if (menuPathOnly === currentPathname) {
      return true
    }
  }

  // 2. 直接路径匹配 - 检查当前路径(不含查询参数)
  if (menuPathOnly === currentPathname) {
    return true
  }

  // 3. 使用 useMatches 检查是否有匹配的路由
  const currentMatches = matches

  // 检查所有当前匹配的路由
  for (const match of currentMatches) {
    // 对于参数化路径，使用正则表达式匹配
    if (menuPathOnly.includes('$')) {
      // 将 $appId 等参数转换为正则表达式模式
      const escapedPath = menuPathOnly.replace(/[.+?^{}()|[\]\\]/g, '\\$&')
      const pathPattern = escapedPath.replace(/\\\$\w+/g, '\\d+')
      const regex = new RegExp(`^${pathPattern}$`)

      // 检查是否匹配当前路径
      if (regex.test(match.pathname)) {
        return true
      }

      // 检查是否是嵌套路由 - 当前路径以菜单路径开头
      // 将菜单路径转换为实际路径进行前缀匹配
      const realPathPattern = menuPathOnly.replace(/\$\w+/g, '\\d+')
      const nestedRegex = new RegExp(`^${realPathPattern}/`)
      if (nestedRegex.test(match.pathname)) {
        return true
      }
    } else {
      // 非参数化路径的直接匹配
      if (match.pathname === menuPathOnly) {
        // 如果菜单路径有查询参数，需要查询参数也匹配
        if (menuQuery) {
          return currentFullPath === menuPathOnly + '?' + menuQuery
        }
        return true
      }

      // 检查是否是嵌套路由 - 确保正确的前缀匹配
      if (match.pathname.startsWith(menuPathOnly + '/')) {
        return true
      }

      // 检查是否是同一个父路径下的不同子页面（如 /send-config/channel 和 /send-config/template）
      // 这种情况适用于标签页式的页面结构
      const menuSegments = menuPathOnly.split('/')
      const currentSegments = match.pathname.split('/')
      
      // 只有当路径段数相同，且路径足够深时才匹配（至少 5 段，如 /admin/email/send-config/channel）
      // 这样可以避免误匹配较短的路径（如 /admin/app/list 和 /admin/app/request 不应该相互匹配）
      // 例如: /admin/email/send-config/channel 和 /admin/email/send-config/template
      // 都有相同的父路径 /admin/email/send-config
      if (menuSegments.length === currentSegments.length && menuSegments.length >= 5) {
        // 比较除了最后一段之外的所有段
        const menuParent = menuSegments.slice(0, -1).join('/')
        const currentParent = currentSegments.slice(0, -1).join('/')
        if (menuParent === currentParent) {
          return true
        }
      }
    }
  }

  return false
}

export function AdminLayout({ className }: AdminLayoutProps) {
  const { isLoading, isError, error, result: menuItems } = useMenu("system")
  const location = useLocation()
  const matches = useMatches()

  // 递归设置菜单项的 isActive 状态
  const setMenuItemActive = useCallback((item: MenuItem): MenuItem => {
    const menuPath = item.path
    const searchString = location.search ? '?' + new URLSearchParams(location.search as any).toString() : ''
    const isCurrentActive = menuPath
      ? isPathActive(menuPath, location.pathname, searchString, matches)
      : false

    // 递归处理子菜单
    const children = item.children?.map(child => setMenuItemActive(child))

    // 检查是否有任何子菜单处于激活状态
    const hasActiveChild = children?.some(child => {
      if (child.isActive) return true
      if (child.children && child.children.length > 0) {
        return child.children.some(c => c.isActive)
      }
      return false
    })

    return {
      ...item,
      isActive: isCurrentActive || hasActiveChild || false,
      children
    }
  }, [location.pathname, location.search, matches])

  // 应用选中逻辑到菜单项
  const enhancedMenuItems = useMemo(() => {
    return (menuItems || []).map(item => setMenuItemActive(item))
  }, [menuItems, setMenuItemActive])

  // 如果侧边栏加载失败，只显示错误信息
  if (isError) {
    return (
      <CenteredError
        variant="layout"
        error={error}
        className={cn("m-4 md:m-0")}
      />
    )
  }




  // 如果侧边栏加载中，显示加载状态
  if (isLoading) {
    return (
      <PageSkeletonTable variant="layout" className={cn("m-4 md:m-0")} />
    )
  }

  // 如果菜单为空，显示无权限提示
  if (!menuItems || menuItems.length === 0) {
    return (
      <CenteredError
        variant="layout"
        error="您当前没有访问后台管理系统的权限，请联系管理员"
        className={cn("m-4 md:m-6")}
      />
    )
  }

  // 正常显示侧边栏和内容区域
  return (
    <SidebarProvider
      defaultOpen={true}
      className={cn(
        'min-h-[calc(100vh-3rem)]',
        // 使用更合理的宽度，w-12 = 48px
        'group-data-[collapsible=icon]:w-12',
        '[&[data-state=collapsed]_[data-sidebar=sidebar]]:!w-12',
        '[&_[data-state=collapsed][data-collapsible=icon]>div:first-child]:!w-12',
        '[&_[data-state=collapsed][data-collapsible=icon]>div:last-child]:!w-12',
        // 使用 3rem 的 CSS 变量值
        '[&[data-state=collapsed]]:![--sidebar-width-icon:3rem]',
        className
      )}
    >
      <Sidebar
        collapsible='icon'
        className={cn(
          "relative h-[calc(100vh-3rem)] hidden md:flex",
          // 使用更宽的侧边栏宽度 w-64 (16rem/256px)
          "!w-64"
        )}
      >
        <SidebarContent className="custom-scrollbar">

          {isLoading ? (
            <SidebarMenuSkeleton count={8} />
          ) : isError ? (
            <SidebarMenuError error={error} />
          ) : (
            <SidebarMenu
              className={className}
              groupLabel="系统管理"
              menuItems={enhancedMenuItems}
            />
          )}
        </SidebarContent>
        <SidebarRail />
      </Sidebar>

      <div className='h-[calc(100vh-3rem)] w-svw overflow-y-auto custom-scrollbar'>

        <Outlet />
      </div>
    </SidebarProvider>
  )
}
