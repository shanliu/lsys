import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@shared/components/ui/dialog'
import { cn } from '@shared/lib/utils'
import React from 'react'

export interface FormDialogProps {
  /** 对话框打开状态 */
  open: boolean
  /** 对话框打开状态变化的回调 */
  onOpenChange: (open: boolean) => void
  /** 对话框标题 */
  title: string
  /** 对话框描述（可选） */
  description?: string
  /** 对话框内容 */
  children: React.ReactNode
  /** 对话框底部操作区（可选） */
  footer?: React.ReactNode
  /** 自定义对话框内容样式 */
  className?: string
  /** 自定义对话框容器样式 */
  contentClassName?: string
}

/**
 * 通用表单对话框组件
 * 提供统一的对话框样式和布局，适用于包含表单的场景
 * 
 * @example
 * ```tsx
 * <FormDialog
 *   open={open}
 *   onOpenChange={setOpen}
 *   title="编辑用户"
 *   description="修改用户信息"
 *   footer={
 *     <>
 *       <Button variant="outline" onClick={() => setOpen(false)}>取消</Button>
 *       <Button onClick={handleSubmit}>保存</Button>
 *     </>
 *   }
 * >
 *   <Form>
 *     <FormField name="name" />
 *   </Form>
 * </FormDialog>
 * ```
 */
export function FormDialog({
  open,
  onOpenChange,
  title,
  description,
  children,
  footer,
  className,
  contentClassName,
}: FormDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className={cn("data-[state=open]:!slide-in-from-left-0 data-[state=closed]:!slide-out-to-left-0 max-w-[calc(100%-2rem)] sm:max-w-lg", contentClassName)}>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          {description && <DialogDescription>{description}</DialogDescription>}
        </DialogHeader>
        <div className={cn('space-y-4', className)}>{children}</div>
        {footer && <DialogFooter>{footer}</DialogFooter>}
      </DialogContent>
    </Dialog>
  )
}
