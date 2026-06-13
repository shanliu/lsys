import { ExtendedFormReturn, FilterFieldContext, LayoutParams } from '../context'
import { Button } from '@shared/components/ui/button'
import { cn } from '@shared/lib/utils'
import { Loader2, Search } from 'lucide-react'
import React from 'react'
import { FieldValues } from 'react-hook-form'

export interface FilterSearchButtonProps<TFieldValues extends FieldValues = FieldValues> {
  form?: ExtendedFormReturn<TFieldValues>
  loading?: boolean
  className?: string
  layoutParams?: LayoutParams
  // 刷新搜索的回调（用于清除缓存并重新加载）
  // 桌面端：双击触发 | 移动端：长按触发
  onRefreshSearch?: () => void
}

export function FilterSearchButton<TFieldValues extends FieldValues = FieldValues>({
  form,
  loading = false,
  className,
  layoutParams,
  onRefreshSearch,
}: FilterSearchButtonProps<TFieldValues>) {
  const filterCtx = React.useContext(FilterFieldContext)
  const effectiveForm = form ?? filterCtx?.form
  const effectiveLayoutParams = layoutParams ?? filterCtx?.layoutParams ?? { isMobile: false }
  const isMobile = effectiveLayoutParams.isMobile

  // 共用的定时器 ref（根据端类型使用不同逻辑）
  const timerRef = React.useRef<NodeJS.Timeout | null>(null)

  // 桌面端：双击检测
  const clickCountRef = React.useRef(0)

  // 移动端：长按检测
  const isPressedRef = React.useRef(false)
  const pressStartTimeRef = React.useRef(0)

  // 桌面端：处理搜索按钮点击（区分单击和双击）
  const handleSearchClick = React.useCallback((e: React.MouseEvent) => {
    e.preventDefault()
    clickCountRef.current += 1

    if (timerRef.current) {
      clearTimeout(timerRef.current)
    }

    timerRef.current = setTimeout(() => {
      if (clickCountRef.current === 1) {
        // 单击：执行正常搜索
        if (effectiveForm) {
          effectiveForm.handleFormSubmit()
        }
      } else if (clickCountRef.current >= 2) {
        // 双击：清除缓存并重新加载，同时触发表单提交以显示顶部加载状态
        if (onRefreshSearch) {
          onRefreshSearch()
        }
        if (effectiveForm) {
          effectiveForm.handleFormSubmit()
        }
      }
      clickCountRef.current = 0
      timerRef.current = null
    }, 250) // 250ms 内的点击视为双击
  }, [effectiveForm, onRefreshSearch])

  // 移动端：处理长按开始
  const handleSearchPressStart = React.useCallback(() => {
    isPressedRef.current = false
    pressStartTimeRef.current = Date.now()

    timerRef.current = setTimeout(() => {
      isPressedRef.current = true
      // 长按：清除缓存并重新加载，同时触发表单提交以显示顶部加载状态
      if (onRefreshSearch) {
        onRefreshSearch()
        // 触觉反馈（如果支持）
        if (navigator.vibrate) {
          navigator.vibrate(50)
        }
      }
      if (effectiveForm) {
        effectiveForm.handleFormSubmit()
      }
    }, 500) // 长按 500ms 触发
  }, [effectiveForm, onRefreshSearch])

  // 移动端：处理长按结束
  const handleSearchPressEnd = React.useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current)
      timerRef.current = null
    }

    // 如果不是长按且按压时间很短，则执行正常搜索
    const pressDuration = Date.now() - pressStartTimeRef.current
    if (!isPressedRef.current && pressDuration < 500 && effectiveForm) {
      effectiveForm.handleFormSubmit()
    }

    isPressedRef.current = false
    pressStartTimeRef.current = 0
  }, [effectiveForm])

  // 清理定时器
  React.useEffect(() => {
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current)
      }
    }
  }, [])

  if (isMobile) {
    return (
      <Button
        onPointerDown={handleSearchPressStart}
        onPointerUp={handleSearchPressEnd}
        onPointerLeave={handleSearchPressEnd}
        onPointerCancel={handleSearchPressEnd}
        disabled={loading}
        variant="outline"
        className={cn("w-full", className)}
      >
        {loading ? (
          <Loader2 className="h-4 w-4 animate-spin" />
        ) : (
          <Search className="h-4 w-4" />
        )}
        搜索
      </Button>
    )
  }

  return (
    <Button
      onClick={handleSearchClick}
      disabled={loading}
      variant="outline"
      size="lg"
      className={cn("w-20", className)}
    >
      {loading ? (
        <Loader2 className="h-4 w-4 animate-spin" />
      ) : (
        <Search className="h-4 w-4" />
      )}
      搜索
    </Button>
  )
}
