
import { SidebarMenu, SidebarMenuError, SidebarMenuSkeleton } from '@apps/main/components/local/sidebar-menu'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { PageSkeletonTable } from '@shared/components/custom/page-placeholder/skeleton-table'
import { Sidebar, SidebarContent, SidebarProvider, SidebarRail } from '@shared/components/ui/sidebar'
import { useMenu } from "@apps/main/hooks/use-menu-data"
import { cn } from '@shared/lib/utils'
import { MenuItem } from '@apps/main/types/ui-menu'
import { Outlet, useLocation, useMatches, useNavigate } from '@tanstack/react-router'
import { useCallback, useEffect, useMemo, useState } from 'react'

interface UserLayoutProps {
  className?: string
}

// 用户布局的路径匹配函数
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
  }

  // 2. 直接路径匹配 - 检查当前路径(不含查询参数)
  if (menuPathOnly === currentPathname) {
    return true
  }

  // 2.1 Prefix match - Check if current pathname starts with menu path (with trailing slash to avoid partial matches like /app/1 vs /app/10)
  if (currentPathname.startsWith(menuPathOnly + '/')) {
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
    }
  }

  return false
}


export function UserLayout({ className }: UserLayoutProps) {
  const { result: menuItems = [], isLoading, isError, error } = useMenu("user")
  const location = useLocation()
  const matches = useMatches()
  const navigate = useNavigate()

  // 获取当前应用ID
  const matchWithAppId = matches.find((m: any) => m.params && m.params.appId)
  const appId = matchWithAppId ? Number((matchWithAppId.params as any).appId) : -1

  // 本地状态：记录访问过的应用IDs
  const [visitedAppIds, setVisitedAppIds] = useState<number[]>([])

  // 当进入一个新的应用时，将其添加到历史记录中
  useEffect(() => {
    if (appId > 0) {
      setVisitedAppIds((prev: number[]) => {
        // 如果已经存在，不用做任何事（保持顺序）
        // 或者移动到最后？通常历史记录是最近的在最后
        if (!prev.includes(appId)) {
          return [...prev, appId]
        }
        return prev
      })
    }
  }, [appId])

  // 关闭应用的处理器
  const handleCloseApp = useCallback((targetAppId: number, e: React.MouseEvent) => {
    // 阻止事件冒泡防止导航跳转
    if (e) {
      e.stopPropagation()
      e.preventDefault()
    }

    setVisitedAppIds((prev: number[]) => {
      const newIds = prev.filter((id: number) => id !== targetAppId)

      // 如果关闭的是当前正在查看的应用，跳转回列表
      if (targetAppId === appId) {
        navigate({ to: '/user/app/list' })
      }
      return newIds
    })
  }, [appId, navigate])



  // 递归处理菜单项，查找并注入应用菜单
  const injectAppMenu = useCallback((items: MenuItem[]): MenuItem[] => {
    return items.map(item => {
      // 找到"应用列表"菜单项
      if (item.path === '/user/app/list') {

        // 构建历史记录子菜单
        const historyChildren: MenuItem[] = visitedAppIds.map((historyAppId: number) => {
          return {
            name: `应用 ${historyAppId}`,
            path: `/user/app/${historyAppId}`,
            icon: item.icon, // 使用父级图标或默认图标
          }
        })

        if (historyChildren.length > 0) {
          return {
            ...item,
            children: [...(item.children || []), ...historyChildren]
          }
        }
      }

      // 递归处理子菜单
      if (item.children) {
        return {
          ...item,
          children: injectAppMenu(item.children)
        }
      }

      return item
    })
  }, [visitedAppIds])

  const handleMenuClose = useCallback((item: MenuItem, e: React.MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()

    if (item.path) {
      const match = item.path.match(/\/user\/app\/(\d+)/)
      if (match && match[1]) {
        handleCloseApp(Number(match[1]), e)
      }
    }
  }, [handleCloseApp])

  // 合并菜单数据
  const mergedMenuItems = useMemo(() => {
    // 如果没有访问记录，直接返回原始菜单
    if (visitedAppIds.length === 0) {
      return menuItems
    }
    return injectAppMenu(menuItems)
  }, [menuItems, visitedAppIds, injectAppMenu])


  // 递归设置菜单项的 isActive 状态
  const setMenuItemActive = useCallback((item: MenuItem): MenuItem => {
    const menuPath = item.path
    let isCurrentActive = false

    if (menuPath) {
      const searchString = location.search ? '?' + new URLSearchParams(location.search as any).toString() : ''
      isCurrentActive = isPathActive(menuPath, location.pathname, searchString, matches)
    }

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



  const isItemClosable = useCallback((item: MenuItem): boolean => {
    if (!item.path) return false
    // 检查路径是否匹配应用详情页模式
    // 注意：只允许关闭在历史记录列表中的项（由 injectAppMenu 生成的项）
    const match = item.path.match(/\/user\/app\/(\d+)/)
    if (match && match[1]) {
      const id = Number(match[1])
      return visitedAppIds.includes(id)
    }
    return false
  }, [visitedAppIds])

  // 应用选中逻辑到菜单项
  const enhancedMenuItems = useMemo(() => {
    return (mergedMenuItems || []).map(item => setMenuItemActive(item))
  }, [mergedMenuItems, setMenuItemActive])

  // 如果侧边栏加载失败，只显示错误信息
  if (isError) {
    return (
      <CenteredError
        variant="layout"
        error={error}
        className={cn('m-4 md:m-0')}
      />
    )
  }

  // 如果侧边栏加载中，显示加载状态
  if (isLoading) {
    return (
      <PageSkeletonTable variant="layout" className={cn('m-4 md:m-0')} />
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
          // 使用 shadcn 默认宽度 w-64 (16rem/256px)
          "!w-64"
        )}
      >
        <SidebarContent className="custom-scrollbar">

          {isLoading ? (
            <SidebarMenuSkeleton count={5} />
          ) : isError ? (
            <SidebarMenuError error={error} />
          ) : (
            <SidebarMenu
              groupLabel="用户管理"
              className={className}
              menuItems={enhancedMenuItems}
              isItemClosable={isItemClosable}
              onItemClose={handleMenuClose}
            />
          )}

        </SidebarContent>
        <SidebarRail />
      </Sidebar>

      <div className='h-[calc(100vh-3rem)] w-svw overflow-y-auto custom-scrollbar'>
        {<Outlet />}
      </div>
    </SidebarProvider>
  )
}
