import { TopNavigationMenu } from '@apps/main/components/local/top-navigation-menu'
import { useMenu } from '@apps/main/hooks/use-menu-data'
import { cn } from '@shared/lib/utils'
import { Route as AppRoute } from '@apps/main/routes/_main/user/app/$appId/route'
import { Outlet, useLocation } from '@tanstack/react-router'
import { useMemo } from 'react'
import { AppDetailSwitcher } from '../ui/app-detail-switcher'
import { UserSidebarNav } from '../ui/user-sidebar-nav'

export function AppDetailLayout() {
  const params = AppRoute.useParams()
  const { result: menuItems = [], isLoading, isError, error } = useMenu("user_app", { appId: params.appId })
  const location = useLocation()

  // 为菜单项设置 isActive 状态
  const enhancedMenuItems = useMemo(() => {
    if (!menuItems || menuItems.length === 0) return []

    // 检查路径是否激活
    const checkIfPathActive = (
      menuPath: string | undefined,
      menuParams?: Record<string, any>,
      noAuthPath?: string
    ): { isActive: boolean; isExactMatch: boolean; matchLength: number } => {
      if (!menuPath && !noAuthPath) {
        return { isActive: false, isExactMatch: false, matchLength: 0 }
      }

      const pathsToCheck: Array<{ path: string; length: number }> = []

      // 添加主路径
      if (menuPath) {
        let expectedPath = menuPath
        if (menuParams) {
          for (const [key, value] of Object.entries(menuParams)) {
            expectedPath = expectedPath.replace(`$${key}`, String(value))
          }
        }
        pathsToCheck.push({ path: expectedPath, length: expectedPath.length })
      }

      // 添加 noAuthPath (检查完整 URL 包含查询参数)
      if (noAuthPath) {
        let expectedNoAuthPath = noAuthPath
        if (menuParams) {
          for (const [key, value] of Object.entries(menuParams)) {
            expectedNoAuthPath = expectedNoAuthPath.replace(`$${key}`, String(value))
          }
        }

        // 分离路径和查询参数
        const [expectedPathPart, expectedQuery] = expectedNoAuthPath.split('?')
        const currentPath = location.pathname

        // 检查路径和查询参数是否都匹配
        if (currentPath === expectedPathPart && expectedQuery) {
          // 检查查询参数是否匹配
          const expectedParams = new URLSearchParams(expectedQuery)

          let queryMatches = true
          expectedParams.forEach((value, key) => {
            const currentValue = location.search?.[key as keyof typeof location.search]
            if (currentValue === undefined || String(currentValue) !== value) {
              queryMatches = false
            }
          })

          if (queryMatches) {
            pathsToCheck.push({ path: expectedPathPart, length: expectedPathPart.length + 1000 })
          }
        }
      }

      // 检查所有路径
      for (const { path: expectedPath, length } of pathsToCheck) {
        // 移除末尾斜杠进行比较
        const normalizedLocation = location.pathname.replace(/\/$/, '')
        const normalizedExpected = expectedPath.replace(/\/$/, '')

        // 精确匹配
        if (normalizedLocation === normalizedExpected) {
          return { isActive: true, isExactMatch: true, matchLength: length }
        }

        // 前缀匹配 - 支持嵌套路由
        // 特殊处理：如果预期路径看起来是"根"路径（以 user/app/数字 结尾），则不进行前缀匹配
        //这是为了防止"应用概览" (/user/app/18) 匹配到同级的其他页面 (/user/app/18/request)
        const isRootAppPath = /\/user\/app\/\d+$/.test(normalizedExpected)

        if (!isRootAppPath && normalizedLocation.startsWith(normalizedExpected + '/')) {
          return { isActive: true, isExactMatch: false, matchLength: length }
        }
      }

      // 同级路径匹配 - 用于 service/* 和 features-* 类型的路径
      // 例如: /user/app/1/service/feature 应该匹配 /user/app/1/service/oauth-client
      // 例如: /user/app/1/features-sms/send 应该匹配 /user/app/1/features-sms/list
      if (menuPath) {
        let expectedPath = menuPath
        if (menuParams) {
          for (const [key, value] of Object.entries(menuParams)) {
            expectedPath = expectedPath.replace(`$${key}`, String(value))
          }
        }

        const expectedParts = expectedPath.split('/').filter(Boolean)
        const currentParts = location.pathname.split('/').filter(Boolean)

        // 至少需要4层路径 (user, app, {id}, service/features-*)
        if (expectedParts.length >= 4 && currentParts.length >= 4) {
          // 检查前4层是否相同 (user/app/{id}/xxx)
          let matchCount = 0
          for (let i = 0; i < 4; i++) {
            if (expectedParts[i] === currentParts[i]) {
              matchCount++
            } else {
              break
            }
          }

          // 如果前4层都匹配，且第4层都是 'service' 或以 'features-' 开头，则认为是同一组
          const expectedFeature = expectedParts[3]
          const currentFeature = currentParts[3]

          if (matchCount === 4 && (
            // service/* 匹配
            (expectedFeature === 'service' && currentFeature === 'service') ||
            // features-* 匹配（匹配相同的 features-xxx 前缀）
            (expectedFeature.startsWith('features-') && expectedFeature === currentFeature)
          )) {
            // 返回较小的匹配长度，这样更具体的路径会优先
            const matchedPath = expectedParts.slice(0, 4).join('/')
            return { isActive: true, isExactMatch: false, matchLength: matchedPath.length }
          }
        }
      }

      return { isActive: false, isExactMatch: false, matchLength: 0 }
    }

    // 递归设置 isActive 状态
    const setActive = (items: typeof menuItems): typeof menuItems => {
      return items.map(item => {
        const matchInfo = checkIfPathActive(item.path, item.params, (item as any).noAuthPath)

        // 递归处理子菜单
        const children = item.children ? setActive(item.children) : undefined

        // 检查子菜单是否有激活的
        const hasActiveChild = children?.some(child => child.isActive)

        return {
          ...item,
          isActive: matchInfo.isActive || hasActiveChild || false,
          children
        }
      })
    }

    const enhanced = setActive(menuItems)
    return enhanced
  }, [menuItems, location])

  return (
    <div className="flex flex-col">
      <UserSidebarNav>
        <div className="flex-1 min-w-0 overflow-visible">
          <TopNavigationMenu
            isLoading={isLoading}
            isError={isError}
            error={error}
            loadingSkeletonCount={3}
            menuItems={enhancedMenuItems}
          />
        </div>
        <div className="flex-shrink-0 ml-auto  ">
          <AppDetailSwitcher userAppId={params.appId} />
        </div>
      </UserSidebarNav>
      <main className={cn("flex-1 px-4 md:p-6 ")}>
        <Outlet />
      </main>
    </div>

  )
}





