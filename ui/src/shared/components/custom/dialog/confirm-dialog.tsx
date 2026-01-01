import React from 'react'
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger } from '@shared/components/ui/alert-dialog'
import { Loader2 } from 'lucide-react'
import { cn } from '@shared/lib/utils'

export interface ConfirmDialogProps {
  /** 子元素作为触发器 */
  children: React.ReactNode
  /** 对话框标题 */
  title: string
  /** 对话框描述内容，支持 React 节点 */
  description: React.ReactNode
  /** 点击确认时的回调函数 */
  onConfirm:  () => Promise<void> | void
  /** 自定义对话框内容样式 */
  className?: string
}

/**
 * 通用确认对话框组件
 * 
 * @example
 * ```tsx
 * <ConfirmDialog
 *   title="确认删除"
 *   description="您确定要删除这个项目吗？删除后无法恢复。"
 *   onConfirm={() => handleDelete()}
 * >
 *   <Button variant="destructive">删除</Button>
 * </ConfirmDialog>
 * ```
 */
export function ConfirmDialog({
  children,
  title,
  description,
  onConfirm,
  className,
}: ConfirmDialogProps) {

  const [loading, setLoading] = React.useState(false)
  return (
    <AlertDialog >
      <AlertDialogTrigger asChild>
        {children}
      </AlertDialogTrigger>
      <AlertDialogContent 
        className={cn("data-[state=open]:!slide-in-from-left-0 data-[state=closed]:!slide-out-to-left-0 max-w-[calc(100%-2rem)] sm:max-w-lg",className)}
      >
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription asChild>
            <div>{description}</div>
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={loading}>
            取消
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={async () => {
              setLoading(true)
              await onConfirm()
              setLoading(false)
            }}
            disabled={loading}
          >
            {loading && <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />}
            <span>确认</span>
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
