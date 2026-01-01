import * as React from "react"
import { Drawer as DrawerPrimitive } from "vaul"
import { X } from "lucide-react"

import { cn } from "@shared/lib/utils"

type DrawerProps = React.ComponentProps<typeof DrawerPrimitive.Root> & {
  /** 抽屉滑出方向，默认从右边滑出 */
  direction?: "top" | "bottom" | "left" | "right"
}

const Drawer = ({
  shouldScaleBackground = false,
  direction = "right",
  children,
  ...props
}: DrawerProps) => (
  <DrawerPrimitive.Root
    shouldScaleBackground={shouldScaleBackground}
    direction={direction}
    {...props}
  >
    {children}
  </DrawerPrimitive.Root>
)
Drawer.displayName = "Drawer"

const DrawerTrigger = DrawerPrimitive.Trigger

const DrawerPortal = DrawerPrimitive.Portal

const DrawerClose = DrawerPrimitive.Close

const DrawerOverlay = React.forwardRef<
  React.ElementRef<typeof DrawerPrimitive.Overlay>,
  React.ComponentPropsWithoutRef<typeof DrawerPrimitive.Overlay>
>(({ className, ...props }, ref) => (
  <DrawerPrimitive.Overlay
    ref={ref}
    className={cn(
      "fixed inset-0 z-50 bg-black/40 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
      className
    )}
    {...props}
  />
))
DrawerOverlay.displayName = DrawerPrimitive.Overlay.displayName

interface DrawerContentProps
  extends React.ComponentPropsWithoutRef<typeof DrawerPrimitive.Content> {
  /** 是否显示关闭按钮，默认显示 */
  showCloseButton?: boolean
  /** 内容区域的自定义类名，可用于覆盖默认的 padding */
  contentClassName?: string
}

const DrawerContent = React.forwardRef<
  React.ElementRef<typeof DrawerPrimitive.Content>,
  DrawerContentProps
>(({ className, children, showCloseButton = true, contentClassName, ...props }, ref) => {
  const contentRef = React.useRef<HTMLDivElement>(null)

  // 阻止在有文字选中时的触摸移动事件传播，防止 vaul 拦截导致无法调整选择范围
  const handleTouchMove = React.useCallback((e: React.TouchEvent) => {
    const selection = window.getSelection()
    if (selection && selection.toString().length > 0) {
      e.stopPropagation()
    }
  }, [])

  // 阻止 wheel 事件传播，防止 vaul 拦截导致无法滚动内部内容（Mac 触摸板）
  const handleWheel = React.useCallback((e: React.WheelEvent) => {
    // 检查事件目标是否在可滚动区域内
    const target = e.target as HTMLElement
    const scrollableParent = target.closest('[data-vaul-no-drag]') || 
                             target.closest('.overflow-y-auto') ||
                             target.closest('.overflow-auto')
    if (scrollableParent) {
      e.stopPropagation()
    }
  }, [])

  return (
    <DrawerPortal>
      <DrawerOverlay />
      <DrawerPrimitive.Content
        ref={ref}
        className={cn(
          "fixed top-0 bottom-0 right-0 z-50 flex h-full flex-col border-l bg-background shadow-lg",
          "w-[95%] md:w-[480px]",
          className
        )}
        style={{
          "--initial-transform": "calc(100% + 8px)",
        } as React.CSSProperties}
        {...props}
      >
        {showCloseButton && (
          <DrawerPrimitive.Close className="absolute right-4 top-4 z-10 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none">
            <X className="h-4 w-4" />
            <span className="sr-only">Close</span>
          </DrawerPrimitive.Close>
        )}
        <div 
          ref={contentRef}
          className={cn("flex-1 overflow-y-auto p-6 custom-scrollbar", contentClassName)}
          onTouchMove={handleTouchMove}
          onWheel={handleWheel}
        >
          {children}
        </div>
      </DrawerPrimitive.Content>
    </DrawerPortal>
  )
})
DrawerContent.displayName = "DrawerContent"

const DrawerHeader = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn("flex flex-col space-y-2 text-center sm:text-left", className)}
    {...props}
  />
)
DrawerHeader.displayName = "DrawerHeader"

const DrawerFooter = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn(
      "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2",
      className
    )}
    {...props}
  />
)
DrawerFooter.displayName = "DrawerFooter"

const DrawerTitle = React.forwardRef<
  React.ElementRef<typeof DrawerPrimitive.Title>,
  React.ComponentPropsWithoutRef<typeof DrawerPrimitive.Title>
>(({ className, ...props }, ref) => (
  <DrawerPrimitive.Title
    ref={ref}
    className={cn("text-lg font-semibold text-foreground", className)}
    {...props}
  />
))
DrawerTitle.displayName = DrawerPrimitive.Title.displayName

const DrawerDescription = React.forwardRef<
  React.ElementRef<typeof DrawerPrimitive.Description>,
  React.ComponentPropsWithoutRef<typeof DrawerPrimitive.Description>
>(({ className, ...props }, ref) => (
  <DrawerPrimitive.Description
    ref={ref}
    className={cn("text-sm text-muted-foreground", className)}
    {...props}
  />
))
DrawerDescription.displayName = DrawerPrimitive.Description.displayName

export {
  Drawer,
  DrawerPortal,
  DrawerOverlay,
  DrawerTrigger,
  DrawerClose,
  DrawerContent,
  DrawerHeader,
  DrawerFooter,
  DrawerTitle,
  DrawerDescription,
}
