import { createCopyWithToast } from '@shared/lib/utils/copy-utils'
import { Copy } from 'lucide-react'
import { useCallback, useRef } from 'react'
import { useToast } from '@shared/contexts/toast-context'
import { cn } from '@shared/lib/utils'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@shared/components/ui/tooltip'

export interface CopyableTextProps {
  value: string
  message?: string
  title?: string
  className?: string
  showIcon?: boolean
  /** If true, show a tooltip with full text when truncated */
  tooltip?: boolean
  /** Force a max width enabling truncation; can pass tailwind class like `max-w-[140px]` */
  maxWidthClassName?: string
}

// 简化版复制组件：只负责展示和复制
export default function CopyableText({
  value,
  message = '已复制',
  title = '点击复制',
  className,
  showIcon = true,
  tooltip = true,
  maxWidthClassName,
}: CopyableTextProps) {
  const { success: showSuccess, error: showError } = useToast()
  const copyFn = createCopyWithToast(showSuccess, showError)
  const textRef = useRef<HTMLSpanElement | null>(null)

  const handleClick = useCallback(() => {
    if (!value) return
    copyFn(value, message)
  }, [value, message, copyFn])

  const core = (
    <code
      className={cn(
        'inline-flex items-center rounded bg-muted px-[0.3rem] py-[0.1rem] text-xs cursor-pointer hover:bg-muted/80 transition-colors select-none max-w-full',
        className,
        maxWidthClassName,
      )}
      onClick={handleClick}
      title={title}
    >
      <span
        ref={textRef}
        className={cn('truncate', showIcon && 'pr-0.5', 'max-w-full')}
      >
        {value}
      </span>
      {showIcon && <Copy className={cn("shrink-0 ml-0.5 h-3 w-3 opacity-60")} />}
    </code>
  )

  if (!tooltip) return core

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>{core}</TooltipTrigger>
        <TooltipContent side="top" sideOffset={4} className={cn("max-w-xs break-all")}>
          {value}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
