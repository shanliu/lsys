import { AlertDialog, AlertDialogAction, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from '@shared/components/ui/alert-dialog'
import { cn } from '@shared/lib/utils'
import React from 'react'

export interface ContentDialogProps {
  /** 子元素作为触发器 */
  children: React.ReactNode
  /** 对话框标题 */
  title: string
  /** 要展示的内容 */
  content: React.ReactNode
  /** 自定义对话框内容样式 */
  className?: string
  /** 确认按钮文字，默认为"确定" */
  confirmText?: string
  /** 自定义 footer，如果提供则覆盖默认的确认按钮 */
  footer?: (closeDialog: () => void) => React.ReactNode
}

/**
 * 通用内容展示对话框组件
 * 直接展示传入的内容
 * 
 * @example
 * ```tsx
 * // 展示内容
 * <ContentDialog
 *   title="用户详情"
 *   content={<div>{user.name}</div>}
 * >
 *   <Button>查看详情</Button>
 * </ContentDialog>
 * 
 * 
 * // 展示错误状态
 * <ContentDialog
 *   title="加载失败"
 *   content={<CenteredError error={error} onReset={retry} />}
 * >
 *   <Button>查看详情</Button>
 * </ContentDialog>
 * ```
 */
export function ContentDialog({
  children,
  title,
  content,
  className,
  confirmText = "确定",
  footer,
}: ContentDialogProps) {
  const [open, setOpen] = React.useState(false)

  const closeDialog = () => setOpen(false)

  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <AlertDialogTrigger asChild>
        {children}
      </AlertDialogTrigger>
      <AlertDialogContent
        className={cn("data-[state=open]:!slide-in-from-left-0 data-[state=closed]:!slide-out-to-left-0 max-w-[calc(100%-2rem)] sm:max-w-lg", className)}
      >
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription asChild>
            <div>
              {content}
            </div>
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          {footer ? (
            footer(closeDialog)
          ) : (
            <AlertDialogAction onClick={closeDialog}>
              {confirmText}
            </AlertDialogAction>
          )}
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
