import { SmartSidebarTrigger } from '@apps/main/components/local/smart-sidebar-trigger'
import { Separator } from '@shared/components/ui/separator'
import { cn } from '@shared/lib/utils'
interface Props {
  children: React.ReactNode
  className?: string
}

export function UserSidebarNav({ children, className }: Props) {
  return (
    <div className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 border-border/40">
      <div className={cn("flex flex-wrap items-center px-4 sm:px-6 py-3 gap-2 sm:gap-4", className)}>
        {/* 左侧按钮组 - 移动端隐藏 */}
        <div className="hidden md:flex items-center gap-2 sm:gap-4 flex-shrink-0">
          <SmartSidebarTrigger />
          <Separator orientation="vertical" className={cn("!h-8 hidden lg:block")} />
        </div>
        {children}
      </div>
    </div>
  )
}
