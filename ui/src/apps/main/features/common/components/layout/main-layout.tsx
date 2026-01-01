import { MobileMenuProvider } from '@apps/main/contexts/mobile-menu-context'
import { MainNavInfo, MainNavMenu } from '@apps/main/features/common/components/ui/main-nav'
import { Outlet } from '@tanstack/react-router'
import React from 'react'


export function MainLayout(): React.ReactElement {
  // 正常渲染
  return (
    <MobileMenuProvider>
      <div className="h-12 flex items-center relative z-50 border-b-gray-400 border-b bg-background">
        <MainNavMenu />
        <MainNavInfo />
      </div>
      <div className="relative z-0 min-h-[calc(100vh-3rem)] bg-background">
        <Outlet />
      </div>
    </MobileMenuProvider>
  )
}
