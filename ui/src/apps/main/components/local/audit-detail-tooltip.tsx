import { Badge } from '@shared/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@shared/components/ui/tooltip'
import { cn } from '@shared/lib/utils'
import { CheckCircle, Database, Key, Shield, XCircle } from 'lucide-react'
import { useState } from 'react'

// 通用的审计详情项类型（兼容两种数据结构）
export interface AuditDetailItem {
  /** 审计结果: 1-授权通过, 0-授权拒绝 / allow-授权通过, deny-授权拒绝 */
  check_result?: string | null
  /** 资源类型 */
  res_type?: string | null
  /** 资源数据 */
  res_data?: string | null
  /** 操作键 */
  op_key?: string | null
  /** 角色数据 */
  role_data?: string | null
}

interface AuditDetailTooltipProps {
  /** 审计详情列表 */
  details: AuditDetailItem[] | null | undefined
  className?: string
}

/**
 * 审计详情 Tooltip 组件
 * 显示审计详情数量，鼠标悬停/点击时显示详细信息
 */
export function AuditDetailTooltip({ details, className }: AuditDetailTooltipProps) {
  const [isHovering, setIsHovering] = useState(false)
  const [clickState, setClickState] = useState<'none' | 'locked'>('none')
  const [lockedOpen, setLockedOpen] = useState(false)

  // 如果没有详情数据，显示占位符
  if (!details || details.length === 0) {
    return (
      <Badge variant="outline" className={cn("text-muted-foreground", className)}>
        0 项
      </Badge>
    )
  }

  // 处理鼠标进入
  const handleMouseEnter = () => {
    setIsHovering(true)
  }

  // 处理鼠标离开
  const handleMouseLeave = () => {
    setIsHovering(false)
    // 如果是锁定状态且鼠标离开，则关闭并重置为非锁定状态
    if (clickState === 'locked') {
      setClickState('none')
      setLockedOpen(false)
    }
  }

  // 处理点击事件
  const handleClick = () => {
    if (clickState === 'none') {
      // 第一次点击：进入锁定状态，保持打开
      setClickState('locked')
      setLockedOpen(true)
    } else {
      // 在锁定状态下：切换打开/关闭
      setLockedOpen(!lockedOpen)
    }
  }

  // 计算tooltip是否应该显示
  const shouldShow = clickState === 'locked' ? lockedOpen : isHovering

  // 判断授权结果
  const isAllowed = (result?: string | null) => {
    if (!result) return false
    return result === '1' || result.toLowerCase() === 'allow'
  }

  return (
    <TooltipProvider delayDuration={200}>
      <Tooltip open={shouldShow}>
        <TooltipTrigger asChild>
          <span
            className={cn("cursor-pointer select-none", className)}
            onClick={handleClick}
            onMouseEnter={handleMouseEnter}
            onMouseLeave={handleMouseLeave}
          >
            <Badge 
              variant="outline" 
              className={cn(
                "hover:bg-accent transition-colors",
                shouldShow && "bg-accent"
              )}
            >
              {details.length} 项
            </Badge>
          </span>
        </TooltipTrigger>
        <TooltipContent
          side="top"
          className={cn(
            "w-auto max-w-[400px] p-0 bg-background border border-border shadow-lg text-foreground pointer-events-auto"
          )}
          sideOffset={8}
          onPointerDownOutside={(e) => e.preventDefault()}
        >
          <div className={cn("max-h-[300px] overflow-y-auto")}>
            {/* 标题 */}
            <div className={cn("sticky top-0 bg-muted/50 px-3 py-2 border-b text-sm font-medium")}>
              权限检查详情 ({details.length} 项)
            </div>
            
            {/* 详情列表 */}
            <div className={cn("divide-y divide-border")}>
              {details.map((item, index) => (
                <div key={index} className={cn("px-3 py-2 space-y-1.5")}>
                  {/* 授权结果 */}
                  <div className={cn("flex items-center gap-2")}>
                    {isAllowed(item.check_result) ? (
                      <CheckCircle className={cn("h-3.5 w-3.5 text-green-500 shrink-0")} />
                    ) : (
                      <XCircle className={cn("h-3.5 w-3.5 text-red-500 shrink-0")} />
                    )}
                    <span className={cn(
                      "text-xs font-medium",
                      isAllowed(item.check_result) ? "text-green-600" : "text-red-600"
                    )}>
                      {isAllowed(item.check_result) ? '授权通过' : '授权失败'}
                    </span>
                  </div>

                  {/* 资源信息 */}
                  {(item.res_type || item.res_data) && (
                    <div className={cn("flex items-start gap-2")}>
                      <Database className={cn("h-3 w-3 text-muted-foreground shrink-0 mt-0.5")} />
                      <div className={cn("text-xs min-w-0 flex-1")}>
                        <span className={cn("text-muted-foreground")}>资源: </span>
                        {item.res_type && (
                          <Badge variant="secondary" className={cn("text-[10px] px-1 py-0 mr-1")}>
                            {item.res_type}
                          </Badge>
                        )}
                        {item.res_data && (
                          <span className={cn("font-mono text-foreground break-all")}>
                            {item.res_data}
                          </span>
                        )}
                      </div>
                    </div>
                  )}

                  {/* 操作信息 */}
                  {item.op_key && (
                    <div className={cn("flex items-center gap-2")}>
                      <Key className={cn("h-3 w-3 text-muted-foreground shrink-0")} />
                      <span className={cn("text-xs")}>
                        <span className={cn("text-muted-foreground")}>操作: </span>
                        <code className={cn("font-mono text-foreground bg-muted px-1 py-0.5 rounded text-[10px]")}>
                          {item.op_key}
                        </code>
                      </span>
                    </div>
                  )}

                  {/* 角色信息 */}
                  {item.role_data && (
                    <div className={cn("flex items-start gap-2")}>
                      <Shield className={cn("h-3 w-3 text-muted-foreground shrink-0 mt-0.5")} />
                      <span className={cn("text-xs min-w-0 flex-1")}>
                        <span className={cn("text-muted-foreground")}>授权角色: </span>
                        <span className={cn("font-mono text-foreground break-all text-[10px]")}>
                          {item.role_data}
                        </span>
                      </span>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
