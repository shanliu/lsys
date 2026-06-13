import { FilterFieldContext, LayoutParams } from '../context'
import { cn } from '@shared/lib/utils'
import React from 'react'

export interface FilterActionsProps {
  children?: React.ReactNode
  className?: string
  layoutParams?: LayoutParams
}

/**
 * 过滤器动作布局容器
 * 根据 layoutParams.isMobile 决定竖排（移动端）或横排（桌面端）
 * 子组件可以是 FilterSearchButton、FilterResetButton 或任意外部按钮（如 ExportButton）
 */
export function FilterActions({ children, className, layoutParams }: FilterActionsProps) {
  const filterCtx = React.useContext(FilterFieldContext)
  const effectiveLayoutParams = layoutParams ?? filterCtx?.layoutParams ?? { isMobile: false }
  const isMobile = effectiveLayoutParams.isMobile

  if (isMobile) {
    return (
      <div className={cn("flex flex-col gap-2 w-full", className)}>
        {children}
      </div>
    )
  }

  return (
    <div className={cn("flex items-center gap-2 flex-shrink-0", className)}>
      {children}
    </div>
  )
}
