import type { MenuConfig } from '@apps/main/hooks/menu-data/types'

import { PageBreadcrumbHeader } from '@apps/main/components/local/page-breadcrumb-header'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { useMenu } from '@apps/main/hooks/use-menu-data'
import { cn } from '@shared/lib/utils'
import { Outlet, useMatches } from '@tanstack/react-router'
import { useMemo } from 'react'

export function UserBreadcrumbLayout() {
  const matches = useMatches()
  const { result: menuData, isLoading, isError, error } = useMenu('user')

  const breadcrumbData = useMemo(() => {
    // 从 matches 获取当前路径
    const pathname = (matches[matches.length - 1] as any)?.pathname || '/'

    // DFS 查找最佳匹配的菜单项
    const result = { best: null as { parents: MenuConfig[]; node: MenuConfig; matchLen: number } | null }

    const dfs = (items: MenuConfig[], parents: MenuConfig[], targetPath: string, matchMode: 'prefix' | 'startsWith') => {
      for (const it of items) {
        const pathWithoutQuery = (it.path || '').split('?')[0]
        const isMatch = matchMode === 'prefix'
          ? Boolean(pathWithoutQuery && targetPath.startsWith(pathWithoutQuery))
          : Boolean(pathWithoutQuery && pathWithoutQuery.startsWith(targetPath))

        if (isMatch) {
          const matchLen = matchMode === 'prefix' ? pathWithoutQuery.length : targetPath.length
          if (!result.best || matchLen > result.best.matchLen) {
            result.best = { parents: [...parents], node: it, matchLen }
          }
        }
        if (it.children?.length) {
          dfs(it.children, [...parents, it], targetPath, matchMode)
        }
      }
    }

    dfs(menuData, [], pathname, 'prefix')

    // 未匹配时尝试父路径
    if (!result.best && pathname !== '/') {
      const lastSlashIndex = pathname.lastIndexOf('/')
      if (lastSlashIndex > 0) {
        dfs(menuData, [], pathname.substring(0, lastSlashIndex), 'startsWith')
      }
    }

    if (!result.best) return []
    return [...result.best.parents, result.best.node].map(i => ({ name: i.name }))
  }, [menuData, matches])

  // 如果面包屑数据加载失败，显示错误在右侧内容区域
  if (isError) {
    return <div className='flex flex-col'><CenteredError
      variant="inline"
      error={error}
      className={cn('m-4 md:m-6')}
    /></div>

  }

  // 如果面包屑数据加载中，显示加载状态在右侧内容区域
  if (isLoading) {
    return (
      <div className='flex flex-col'>
        <CenteredLoading variant="card" className={cn('m-4 md:m-6')} />
      </div>
    )
  }

  // 正常显示面包屑和内容
  return (
    <div className='flex flex-col'>
      <PageBreadcrumbHeader breadcrumbData={breadcrumbData} />
      {<Outlet />}
    </div>
  )
}
