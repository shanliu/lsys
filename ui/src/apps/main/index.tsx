
import { Config } from '@shared/lib/config'
import { queryClient } from '@shared/lib/query-client'
import '@shared/styles/globals.css'
import { QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider, createRouter } from '@tanstack/react-router'
import { StrictMode } from 'react'
import * as ReactDOM from 'react-dom/client'
import { routeTree } from './routeTree.gen'
import './styles/app.css'

// Create a new router instance
const router = createRouter({
  routeTree,
  basepath: Config.appBaseUrl,
  context: { queryClient },
  defaultPreload: 'viewport',
  defaultPreloadStaleTime: 0,
  defaultPreloadGcTime: 5 * 60 * 1000, // 5 minutes
})

// Register the router instance for type safety
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

// Render the app
// 创建 root 元素（如果不存在）
let rootElement = document.getElementById('root')
if (!rootElement) {
  rootElement = document.createElement('div')
  rootElement.id = 'root'
  document.body.appendChild(rootElement)
}

if (rootElement) {
  const root = ReactDOM.createRoot(rootElement)
  root.render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </StrictMode>
  )
}
