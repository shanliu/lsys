import { ClearableInput } from '@shared/components/custom/input/clearable-input'
import { LayoutParams } from '@apps/main/components/filter-container/container'
import { cn } from '@shared/lib/utils'
import React from 'react'
import { FieldPath, FieldValues, useController } from 'react-hook-form'

export interface FilterInputProps<
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
> {
  name: TName
  placeholder: string
  type?: 'text' | 'number'
  className?: string
  label: string
  disabled?: boolean
  layoutParams?: LayoutParams
  rules?: Parameters<typeof useController<TFieldValues, TName>>[0]['rules']
}

/**
 * 过滤器输入组件
 * 自动使用 react-hook-form 进行状态管理，支持移动端和桌面端响应式布局
 */
export function FilterInput<
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
>({
  name,
  placeholder,
  type = 'text',
  className,
  label,
  disabled = false,
  layoutParams,
  rules,
}: FilterInputProps<TFieldValues, TName>) {
  const {
    field: { value, onChange },
    fieldState: { error }
  } = useController({
    name,
    rules,
  })

  const handleChange = React.useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const inputValue = e.target.value

    if (type === 'number') {
      const trimmedValue = inputValue.trim()
      if (trimmedValue === '') {
        onChange(undefined)
      } else {
        const num = Number(trimmedValue)
        onChange(Number.isNaN(num) ? undefined : num)
      }
    } else {
      onChange(inputValue || undefined)
    }
  }, [onChange, type])

  const handleClear = React.useCallback(() => {
    onChange(type === 'number' ? undefined : '')
  }, [onChange, type])

  // 移动端显示标签布局，桌面端显示无标签布局
  if (layoutParams?.isMobile) {
    return (
      <div className={cn("flex items-center gap-3 w-full min-w-0", className)}>
        <div className="text-xs font-medium text-muted-foreground leading-none flex-shrink-0 w-16">
          {label}
        </div>
        <div className="relative flex-1 min-w-0">
          <ClearableInput
            value={type === 'number' ? (value ? String(value) : '') : (value || '')}
            placeholder={placeholder}
            onChange={handleChange}
            onClear={handleClear}
            enableDoubleClickPaste={true}
            showClearButton={true}
            disabled={disabled}
            className={cn(
              "h-9 text-sm shadow-none", // 移动端：较高且小字体，移除阴影
              error && "border-destructive focus-visible:ring-destructive"
            )}
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
    <div className={cn(
      "flex flex-col", // 桌面端：垂直布局，由外层容器控制宽度
      className
    )}>
      <div className="text-sm font-medium text-muted-foreground mb-3">
        {label}
      </div>
      <div className="relative">
        <ClearableInput
          value={type === 'number' ? (value ? String(value) : '') : (value || '')}
          placeholder={placeholder}
          onChange={handleChange}
          onClear={handleClear}
          enableDoubleClickPaste={true}
          showClearButton={true}
          disabled={disabled}
          className={cn(
            "shadow-none", // 移除阴影
            layoutParams?.isMobile
              ? "h-9 text-sm" // 移动端：较高且小字体
              : "h-10 text-base", // 桌面端：标准高度和字体
            error && "border-destructive focus-visible:ring-destructive"
          )}
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
