import { LayoutParams } from '@apps/main/components/filter-container/container'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shared/components/ui/select'
import { cn } from '@shared/lib/utils'
import React from 'react'
import { FieldPath, FieldValues, useController } from 'react-hook-form'

export interface FilterSelectOption {
  value: string
  label: string
}

export interface FilterSelectProps<
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
> {
  name: TName
  placeholder: string
  options: FilterSelectOption[]
  className?: string
  label: string
  disabled?: boolean
  allLabel?: string  // 有值就显示"全部"选项，没有就不显示
  layoutParams?: LayoutParams
  rules?: Parameters<typeof useController<TFieldValues, TName>>[0]['rules']
}

/**
 * 过滤器选择组件
 * 自动使用 react-hook-form 进行状态管理，支持移动端和桌面端响应式布局
 */
export function FilterSelect<
  TFieldValues extends FieldValues = FieldValues,
  TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
>({
  name,
  placeholder,
  options,
  className,
  label,
  disabled = false,
  allLabel,
  layoutParams,
  rules,
}: FilterSelectProps<TFieldValues, TName>) {
  const {
    field: { value, onChange },
    fieldState: { error }
  } = useController({
    name,
    rules,
  })

  const handleValueChange = React.useCallback((selectedValue: string) => {
    // 如果选择了 "全部" 选项（值为 "__all__"），则返回 undefined
    onChange(selectedValue === '__all__' ? "" : selectedValue)
  }, [onChange])

  // 确保 value 始终是字符串类型，用于受控组件
  const selectValue = value || '__all__'

  // 移动端显示标签布局，桌面端显示无标签布局
  if (layoutParams?.isMobile) {
    return (
      <div className={cn("flex items-center gap-3 w-full min-w-0", className)}>
        <div className="text-xs font-medium text-muted-foreground leading-none flex-shrink-0 w-16">
          {label}
        </div>
        <div className="relative flex-1 min-w-0">
          <Select
            key={`${name}-${selectValue}`}
            value={selectValue}
            onValueChange={handleValueChange}
            disabled={disabled}
          >
            <SelectTrigger className={cn(
              "h-9 shadow-none", // 移动端：较高，移除阴影
              (value === '__all__' || !value) && "[&>span]:text-sm [&>span]:text-muted-foreground", // 选中"全部"时的样式
              error && "border-destructive focus-visible:ring-destructive"
            )}>
              <SelectValue placeholder={placeholder} />
            </SelectTrigger>
            <SelectContent className="max-h-[300px]">
              {allLabel && <SelectItem value="__all__">{allLabel}</SelectItem>}
              {options.map((option: FilterSelectOption) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
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
      <Select
        key={`${name}-${selectValue}`}
        value={selectValue}
        onValueChange={handleValueChange}
        disabled={disabled}
      >
        <SelectTrigger className={cn(
          "h-10 shadow-none", // 桌面端：标准高度，移除阴影
          (value === '__all__' || !value) && "[&>span]:text-sm [&>span]:text-muted-foreground", // 选中"全部"时的样式
          error && "border-destructive focus-visible:ring-destructive"
        )}>
          <SelectValue placeholder={placeholder} />
        </SelectTrigger>
        <SelectContent className="max-h-[300px]">
          {allLabel && <SelectItem value="__all__">{allLabel}</SelectItem>}
          {options.map((option: FilterSelectOption) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {error && (
        <div className="text-xs text-destructive mt-1">
          {error.message}
        </div>
      )}
    </div>
  )
}
