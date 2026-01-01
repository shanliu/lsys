import React from 'react'
import { cn } from '@shared/lib/utils'
import { Badge } from '@shared/components/ui/badge'
import { BarChart3, Loader2 } from 'lucide-react'
import { useIsMobile } from '@shared/hooks/use-mobile'

// 总数显示组件的 props 接口
export interface FilterTotalCountProps {
  // 总数值
  total: number
  // 总数标签文本，默认为 "总数"
  label?: string
  // 自定义样式类名
  className?: string
  // 是否正在加载
  loading?: boolean
}

/**
 * 总数显示组件
 * 纯展示组件，只负责显示数据和样式
 * 定位由父容器（FilterContainer）控制
 * 
 * 支持加载状态：
 * - 加载时显示旋转的加载图标和省略号
 * - 加载时组件会有半透明效果
 */
export function FilterTotalCount({
  total,
  label = '总数',
  className,
  loading = false,
}: FilterTotalCountProps) {
  const isMobile = useIsMobile()

  // 格式化总数显示
  const formattedTotal = React.useMemo(() => {
    if (total >= 10000) {
      return `${(total / 10000).toFixed(1)}万`
    }
    if (total >= 1000) {
      return `${(total / 1000).toFixed(1)}k`
    }
    return total.toString()
  }, [total])

  // 根据设备类型使用不同的显示方式
  if (isMobile) {
    // 移动端：简洁文本显示，无背景
    return (
      <div className={cn(
        "flex items-center gap-1.5 text-sm text-muted-foreground",
        loading && "opacity-70",
        className
      )}>
        <BarChart3 className={cn("h-3.5 w-3.5 text-primary")} />
        <span>{label}:</span>
        <span className="font-medium text-foreground">
          {loading ?  <Loader2 className={cn("h-3.5 w-3.5 animate-spin text-primary")} /> : formattedTotal}
        </span>
      </div>
    )
  }

  // 桌面端：Badge 样式
  return (
    <Badge 
      variant="secondary" 
      className={cn(
        "flex items-center gap-1 font-medium text-xs bg-background/90 backdrop-blur-sm text-foreground border border-border/50 shadow-sm hover:bg-background/95 transition-colors px-2 py-1",
        loading && "opacity-70",
        className
      )}
    >
      <BarChart3 className={cn("h-3 w-3 text-primary")} />
      <span className="text-muted-foreground">{label}:</span>
      <span className="font-semibold text-primary">
        {loading ?   <Loader2 className={cn("h-3 w-3 animate-spin text-primary")} /> : formattedTotal}
      </span>
    </Badge>
  )
}
