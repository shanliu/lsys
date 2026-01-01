import { LayoutParams } from '@apps/main/components/filter-container/container'
import { SystemAppSelector, type SystemAppSelectorProps } from '@apps/main/components/selector/system-app-selector'
import { cn } from '@shared/lib/utils'
import React from 'react'
import { FieldPath, FieldValues, useController } from 'react-hook-form'

export interface FilterSystemAppSelectorProps<
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
> {
  name: TName
  placeholder: string
  className?: string
  label: string
  disabled?: boolean
  layoutParams?: LayoutParams
  rules?: Parameters<typeof useController<TFieldValues, TName>>[0]['rules']
  /** SystemAppSelector 的其他配置选项 */
  appSelectorProps?: Omit<SystemAppSelectorProps, 'value' | 'onValueChange' | 'placeholder' | 'disabled' | 'triggerClassName' | 'layoutParams'>
}

/**
 * 过滤器系统应用选择组件
 * 自动使用 react-hook-form 进行状态管理，支持移动端和桌面端响应式布局
 */
export function FilterSystemAppSelector<
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
>({
  name,
  placeholder,
  className,
  label,
  disabled = false,
  layoutParams,
  rules,
  appSelectorProps,
}: FilterSystemAppSelectorProps<TFieldValues, TName>) {
  const {
    field: { value, onChange },
    fieldState: { error }
  } = useController({
    name,
    rules,
  })

  const handleValueChange = React.useCallback((appId: number | undefined) => {
    // 将数字转换为字符串存储在表单中，因为大多数表单 schema 期望字符串
    // 使用空字符串而不是 undefined，以确保 react-hook-form 正确处理清除操作
    const newValue = appId !== undefined ? String(appId) : ''
    onChange(newValue)
  }, [onChange])

  // 将字符串值转换为数字传递给 SystemAppSelector
  const numericValue = React.useMemo(() => {
    if (value === undefined || value === null || value === '') {
      return undefined
    }
    const parsed = Number(value)
    return isNaN(parsed) ? undefined : parsed
  }, [value])

  // 移动端显示标签布局，桌面端显示无标签布局
  if (layoutParams?.isMobile) {
    return (
      <div className={cn('flex items-center gap-3 w-full min-w-0', className)}>
        <div className="text-xs font-medium text-muted-foreground leading-none flex-shrink-0 w-16">
          {label}
        </div>
        <div className="relative flex-1 min-w-0">
          <SystemAppSelector
            value={numericValue}
            onValueChange={handleValueChange}
            placeholder={placeholder}
            disabled={disabled}
            triggerClassName="w-full shadow-none"
            showClearButton={true}
            layoutParams={layoutParams}
            {...appSelectorProps}
          />
          {error && (
            <div className="text-xs text-destructive mt-1">
              {error.message}
            </div>
          )}
        </div>
      </div>
    )
  }

  // 桌面端垂直标签布局
  return (
    <div className={cn('flex-shrink-0 w-full', className)}>
      <div className="text-sm font-medium text-muted-foreground mb-3">
        {label}
      </div>
      <SystemAppSelector
        value={numericValue}
        onValueChange={handleValueChange}
        placeholder={placeholder}
        disabled={disabled}
        triggerClassName="w-full shadow-none"
        showClearButton={true}
        layoutParams={layoutParams}
        {...appSelectorProps}
      />
      {error && (
        <div className="text-xs text-destructive mt-1">
          {error.message}
        </div>
      )}
    </div>
  )
}
