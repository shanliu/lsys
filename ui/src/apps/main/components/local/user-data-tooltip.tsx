import { Badge } from '@shared/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@shared/components/ui/tooltip'
import { cn } from '@shared/lib/utils'
import { type UserDataType } from '@shared/types/base-schema'
import { Building2, Database, ExternalLink, Hash, Shield, User } from 'lucide-react'
import { useState } from 'react'

interface UserDataTooltipProps {
  userData: UserDataType | null | undefined
  className?: string
}

export function UserDataTooltip({ userData, className }: UserDataTooltipProps) {
  const [isHovering, setIsHovering] = useState(false)
  const [clickState, setClickState] = useState<'none' | 'locked'>('none')
  const [lockedOpen, setLockedOpen] = useState(false)

  // 如果没有用户数据，显示占位符
  if (!userData) {
    return (
      <span className={cn("text-muted-foreground", className)}>
        -
      </span>
    )
  }

  // 判断是否为系统内用户
  const isSystemUser = userData.app_id === 0

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

  return (
    <TooltipProvider delayDuration={200}>
      <Tooltip open={shouldShow}>
        <TooltipTrigger asChild>
          <span
            className={cn("text-primary hover:text-primary/80 select-none cursor-pointer", className)}
            onClick={handleClick}
            onMouseEnter={handleMouseEnter}
            onMouseLeave={handleMouseLeave}
          >
            <span className={cn("font-medium truncate max-w-[80px]")} title={userData.user_nickname}>
              {userData.user_nickname}
            </span>
          </span>
        </TooltipTrigger>
        <TooltipContent
          side="top"
          className={cn("w-80 p-4 bg-background border border-border shadow-lg text-foreground pointer-events-auto")}
          sideOffset={8}
          onPointerDownOutside={(e) => e.preventDefault()}
        >
          <div className={cn("space-y-3")}>
            {/* 用户类型和昵称 */}
            <div className={cn("flex items-start gap-3")}>

              <div className={cn("min-w-0 flex-1")}>
                <div className={cn("font-medium text-sm text-foreground break-words")}>
                  {userData.user_nickname}
                </div>
              </div>
              {isSystemUser ? (
                <Badge variant="secondary" className={cn("text-xs shrink-0")}>
                  <Shield className={cn("h-3 w-3 mr-1")} />
                  系统用户
                </Badge>
              ) : (
                <Badge variant="outline" className={cn("text-xs shrink-0")}>
                  <User className={cn("h-3 w-3 mr-1")} />
                  外部用户
                </Badge>
              )}
            </div>

            {/* 账号信息 */}
            <div className={cn("text-sm")}>
              <span className={cn("text-muted-foreground")}>账号: </span>
              <span className={cn("font-mono text-foreground break-all")}>{userData.user_account}</span>
            </div>

            {/* ID和应用信息 */}
            <div className={cn("space-y-2")}>
              <div className={cn("flex items-center gap-2")}>
                <Hash className={cn("h-3 w-3 text-muted-foreground shrink-0")} />
                <span className={cn("text-muted-foreground text-sm")}>ID: </span>
                <span className={cn("text-foreground text-sm")}>{userData.id}</span>
              </div>
              <div className={cn("flex items-center gap-2")}>
                <Building2 className={cn("h-3 w-3 text-muted-foreground shrink-0")} />
                <span className={cn("text-muted-foreground text-sm")}>
                  {isSystemUser ? '系统用户' : `应用: ${userData.app_id} 用户`}
                </span>
              </div>
            </div>

            {/* 用户数据 - 始终显示 */}
            <div className={cn("flex items-center gap-2")}>
              {isSystemUser ? (
                <>
                  <Database className={cn("h-3 w-3 text-muted-foreground")} />
                  <span className={cn("text-sm text-muted-foreground")}>账号ID:</span>
                  <span className={cn("font-mono text-foreground text-sm")}>{userData.user_data}</span>
                </>
              ) : (
                <>
                  <ExternalLink className={cn("h-3 w-3 text-muted-foreground")} />
                  <span className={cn("text-sm text-muted-foreground")}>外部应用标识:</span>
                  <span className={cn("font-mono text-foreground text-sm")}>{userData.user_data}</span>
                </>
              )}
            </div>
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
