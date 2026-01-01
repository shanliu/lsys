

import { SessionExpiredDialog } from '@apps/main/components/local/session-expired-dialog'
import { ThemeProvider } from '@shared/contexts/theme-context'
import { ToastProvider } from '@shared/contexts/toast-context'
import { NavigationProgress } from '@apps/main/features/common/components/ui/navigation-progress'
import { Config } from '@shared/lib/config'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { Outlet } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools'
import * as React from "react"


export function RootLayout(): React.ReactElement {
  return (
    <>
      <ThemeProvider>
        <NavigationProgress />
        <ToastProvider>
          {<Outlet />}
          <SessionExpiredDialog />
        </ToastProvider>
      </ThemeProvider>
      {Config.showDevtools && (
        <>
          <ReactQueryDevtools buttonPosition='bottom-right' />
          <TanStackRouterDevtools position='bottom-left' />
        </>
      )}
    </>
  )
}


