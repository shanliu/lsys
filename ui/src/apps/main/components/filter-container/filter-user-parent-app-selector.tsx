import { UserParentAppSelector, type UserParentAppSelectorProps } from '@apps/main/components/selector/user-parent-app-selector'
import { cn } from '@shared/lib/utils'
import React from 'react'
import { FieldPath, FieldValues, useController } from 'react-hook-form'
import { LayoutParams } from './container'

export interface FilterUserParentAppSelectorProps<
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
  /** ParentAppSelector 的其他配置选项 */
  appSelectorProps?: Omit<UserParentAppSelectorProps, 'value' | 'onValueChange' | 'placeholder' | 'disabled' | 'triggerClassName'>
}

/**
 * 过滤器应用选择组件
 * 自动使用 react-hook-form 进行状态管理，支持移动端和桌面端响应式布局
 */
export function FilterUserParentAppSelector<
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
}: FilterUserParentAppSelectorProps<TFieldValues, TName>) {
  const {
    field: { value, onChange },
    fieldState: { error }
  } = useController({
    name,
    rules,
  })

  const handleValueChange = React.useCallback((appId: string | undefined) => {
    // 使用空字符串而不是 undefined，以确保 react-hook-form 正确处理清除操作
    const newValue = appId !== undefined ? appId : ''
    onChange(newValue)
  }, [onChange])

  // 移动端显示标签布局，桌面端显示无标签布局
  if (layoutParams?.isMobile) {
    return (
      <div className={cn("flex items-center gap-3 w-full min-w-0", className)}>
        <div className="text-xs font-medium text-muted-foreground leading-none flex-shrink-0 w-16">
          {label}
        </div>
        <div className="relative flex-1 min-w-0">
          <UserParentAppSelector
            value={value}
            placeholder={placeholder}
            onValueChange={handleValueChange}
            disabled={disabled}
            triggerClassName={cn(
              "w-full h-9 text-sm shadow-none", // 移动端：全宽、较高且小字体，移除阴影
              error && "border-destructive focus-visible:ring-destructive",
            )}
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
    <div className={cn(
      "flex-shrink-0 w-full", // 桌面端：100%宽度
      className
    )}>
      <div className="text-sm font-medium text-muted-foreground mb-3">
        {label}
      </div>
      <UserParentAppSelector
        value={value}
        placeholder={placeholder}
        onValueChange={handleValueChange}
        disabled={disabled}
        triggerClassName={cn(
          "w-full shadow-none", // 全宽，移除阴影
          layoutParams?.isMobile
            ? "h-9 text-sm" // 移动端：较高且小字体
            : "h-10 text-base", // 桌面端：标准高度和字体
          error && "border-destructive focus-visible:ring-destructive",
        )}
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
