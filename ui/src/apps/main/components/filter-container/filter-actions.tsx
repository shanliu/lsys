import { ExtendedFormReturn, LayoutParams } from '@apps/main/components/filter-container/container'
import { Button } from '@shared/components/ui/button'
import { cn } from '@shared/lib/utils'
import { Loader2, RotateCcw, Search } from 'lucide-react'
import React from 'react'
import { FieldValues } from 'react-hook-form'

export interface FilterActionsProps<TFieldValues extends FieldValues = FieldValues> {
  // react-hook-form 实例
  form?: ExtendedFormReturn<TFieldValues>
  // 外部数据加载状态
  loading?: boolean
  className?: string
  layoutParams?: LayoutParams
  // 刷新搜索的回调（用于清除缓存并重新加载）
  // 桌面端：双击搜索按钮触发 | 移动端：长按搜索按钮触发
  onRefreshSearch?: () => void
}

/**
 * 过滤器动作组件
 * 提供搜索和清除按钮，支持移动端和桌面端的响应式布局
 * 需要配合 react-hook-form 使用
 * @param loading - 外部数据加载状态，加载时禁用所有按钮
 * @param onRefreshSearch - 桌面端双击或移动端长按搜索按钮时触发（用于清除缓存并重新加载）
 */
export function FilterActions<TFieldValues extends FieldValues = FieldValues>({
  form,
  loading = false,
  className,
  layoutParams,
  onRefreshSearch,
}: FilterActionsProps<TFieldValues>) {
  const isMobile = layoutParams?.isMobile

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
        if (form) {
          form.handleFormSubmit()
        }
      } else if (clickCountRef.current >= 2) {
        // 双击：清除缓存并重新加载
        if (onRefreshSearch) {
          onRefreshSearch()
        }
      }
      clickCountRef.current = 0
      timerRef.current = null
    }, 250) // 250ms 内的点击视为双击
  }, [form, onRefreshSearch])

  // 移动端：处理长按开始
  const handleSearchPressStart = React.useCallback(() => {
    isPressedRef.current = false
    pressStartTimeRef.current = Date.now()

    timerRef.current = setTimeout(() => {
      isPressedRef.current = true
      // 长按：清除缓存并重新加载
      if (onRefreshSearch) {
        onRefreshSearch()
        // 触觉反馈（如果支持）
        if (navigator.vibrate) {
          navigator.vibrate(50)
        }
      }
    }, 500) // 长按 500ms 触发
  }, [onRefreshSearch])

  // 移动端：处理长按结束
  const handleSearchPressEnd = React.useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current)
      timerRef.current = null
    }

    // 如果不是长按且按压时间很短，则执行正常搜索
    const pressDuration = Date.now() - pressStartTimeRef.current
    if (!isPressedRef.current && pressDuration < 500 && form) {
      form.handleFormSubmit()
    }

    isPressedRef.current = false
    pressStartTimeRef.current = 0
  }, [form])

  // 清理定时器
  React.useEffect(() => {
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current)
      }
    }
  }, [])

  const handleClear = React.useCallback(() => {
    if (form) {
      form.handleFormReset();
    }
  }, [form]);

  // 过滤器表单不需要验证，始终视为有效
  const isFormValid = true

  // 检查表单是否有值（判断是否显示重置按钮）
  const hasFormValues = React.useMemo(() => {
    if (!form) return false // 没有 form 则不显示重置按钮

    const values = form.getValues()
    return Object.values(values).some(value => {
      if (value === null || value === undefined || value === '') return false
      if (typeof value === 'string' && value.trim() === '') return false
      if (Array.isArray(value) && value.length === 0) return false
      return true
    })
  }, [form])

  // 监听表单值变化以触发重新渲染
  form?.watch?.()

  // 计算实际的禁用和加载状态
  const actualSearchDisabled = !isFormValid || loading
  const actualSearchLoading = loading
  const actualClearDisabled = loading // 数据加载时也禁用清除按钮
  // 移动端显示水平布局，两个按钮各占50%
  if (isMobile) {
    return (
      <div className={cn(
        "flex gap-2 w-full",
        className
      )}>
        <Button
          onPointerDown={handleSearchPressStart}
          onPointerUp={handleSearchPressEnd}
          onPointerLeave={handleSearchPressEnd}
          onPointerCancel={handleSearchPressEnd}
          disabled={actualSearchDisabled || actualSearchLoading}
          variant="outline"
          className={cn(hasFormValues ? "flex-1" : "w-full")}
        >
          {actualSearchLoading ? (
            <Loader2 className={cn("h-4 w-4 animate-spin")} />
          ) : (
            <Search className={cn("h-4 w-4")} />
          )}
          搜索
        </Button>

        {hasFormValues && (
          <Button
            onClick={handleClear}
            disabled={actualClearDisabled}
            variant="outline"
            className={cn("flex-1")}
          >
            <RotateCcw className={cn("h-4 w-4")} />
            重置
          </Button>
        )}
      </div>
    )
  }

  // 桌面端：水平排列
  return (
    <div className={cn(
      "flex items-center gap-2 flex-shrink-0",
      className
    )}>
      <Button
        onClick={handleSearchClick}
        disabled={actualSearchDisabled || actualSearchLoading}
        variant="outline"
        size="lg"
        className={cn("w-20")}
      >
        {actualSearchLoading ? (
          <Loader2 className={cn("h-4 w-4 animate-spin")} />
        ) : (
          <Search className={cn("h-4 w-4")} />
        )}
        搜索
      </Button>

      {hasFormValues && (
        <Button
          onClick={handleClear}
          disabled={actualClearDisabled}
          variant="outline"
          size="lg"
          className={cn("w-20")}
        >
          <RotateCcw className={cn("h-4 w-4")} />
          重置
        </Button>
      )}
    </div>
  )
}
