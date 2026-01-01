import { Button } from '@shared/components/ui/button'
import { useSidebar } from '@shared/components/ui/sidebar'
import { cn } from '@shared/lib/utils'
import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'

interface SmartSidebarTriggerProps extends React.ComponentProps<typeof Button> {
  className?: string
}

export function SmartSidebarTrigger({ className, onClick, ...props }: SmartSidebarTriggerProps) {
  const { toggleSidebar, state } = useSidebar()

  return (
    <Button
      data-sidebar="trigger"
      data-slot="sidebar-trigger"
      variant="outline"
      size="icon"
      className={cn(
        'hidden md:flex scale-125 lg:scale-100 transition-transform duration-200 size-7 relative',
        className
      )}
      onClick={(event) => {
        onClick?.(event)
        toggleSidebar()
      }}
      {...props}
    >
      {/* 侧边栏打开时显示的图标 (PanelLeftClose) */}
      <PanelLeftClose className={cn(
        "size-4 transition-opacity duration-200 absolute text-muted-foreground",
        state === "expanded" ? "opacity-100" : "opacity-0"
      )} />

      {/* 侧边栏关闭时显示的图标 (PanelLeftOpen) */}
      <PanelLeftOpen className={cn(
        "size-4 transition-opacity duration-200 absolute text-muted-foreground",
        state === "collapsed" ? "opacity-100" : "opacity-0"
      )} />

    </Button>
  )
}
