import { ClearableInput } from '@shared/components/custom/input/clearable-input'
import { LayoutParams } from '@apps/main/components/filter-container/container'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shared/components/ui/select'
import { cn } from '@shared/lib/utils'
import React from 'react'
import { FieldPath, FieldValues, useController } from 'react-hook-form'

export interface ContentSearchOption {
    value: string
    label: string
}

export interface FilterContentSearchProps<
    TFieldValues extends FieldValues = FieldValues,
    TTypeName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TValueName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
> {
    /** 类型字段名（绑定到下拉选择） */
    typeName: TTypeName
    /** 值字段名（绑定到输入框） */
    valueName: TValueName
    /** 下拉选项列表，第一项通常为"无" */
    options: ContentSearchOption[]
    /** 类型选择的占位符 */
    typePlaceholder?: string
    /** 输入框占位符，可根据选中类型动态指定 */
    valuePlaceholder?: string | ((selectedType: string) => string)
    className?: string
    label: string
    disabled?: boolean
    layoutParams?: LayoutParams
    typeRules?: Parameters<typeof useController<TFieldValues, TTypeName>>[0]['rules']
    valueRules?: Parameters<typeof useController<TFieldValues, TValueName>>[0]['rules']
}

/**
 * 复合过滤器：左侧下拉选择类型 + 右侧输入框
 *
 * 当选择"无"（value === ''）时隐藏输入框。
 * 支持移动端和桌面端响应式布局。
 */
export function FilterContentSearch<
    TFieldValues extends FieldValues = FieldValues,
    TTypeName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TValueName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
>({
    typeName,
    valueName,
    options,
    typePlaceholder = '选择类型',
    valuePlaceholder = '请输入...',
    className,
    label,
    disabled = false,
    layoutParams,
    typeRules,
    valueRules,
}: FilterContentSearchProps<TFieldValues, TTypeName, TValueName>) {
    const {
        field: { value: typeValue, onChange: onTypeChange },
        fieldState: { error: typeError },
    } = useController({ name: typeName, rules: typeRules })

    const {
        field: { value: inputValue, onChange: onInputChange },
        fieldState: { error: valueError },
    } = useController({ name: valueName, rules: valueRules })

    const selectedType = typeValue || ''
    const hasType = selectedType !== ''

    const handleTypeChange = React.useCallback(
        (val: string) => {
            const newType = val === '__none__' ? '' : val
            onTypeChange(newType || undefined)
            // 切换类型时清空输入值
            if (!newType) {
                onInputChange(undefined)
            }
        },
        [onTypeChange, onInputChange],
    )

    const handleInputChange = React.useCallback(
        (e: React.ChangeEvent<HTMLInputElement>) => {
            onInputChange(e.target.value || undefined)
        },
        [onInputChange],
    )

    const handleInputClear = React.useCallback(() => {
        onInputChange(undefined)
    }, [onInputChange])

    const selectValue = selectedType || '__none__'

    const resolvedPlaceholder =
        typeof valuePlaceholder === 'function'
            ? valuePlaceholder(selectedType)
            : valuePlaceholder

    const error = typeError || valueError

    // ======================== 移动端 ========================
    if (layoutParams?.isMobile) {
        return (
            <div className={cn('flex flex-col gap-2 w-full min-w-0', className)}>
                {/* 类型选择行 */}
                <div className="flex items-center gap-3 w-full min-w-0">
                    <div className="text-xs font-medium text-muted-foreground leading-none flex-shrink-0 w-16">
                        {label}
                    </div>
                    <div className="relative flex-1 min-w-0">
                        <Select
                            key={`${String(typeName)}-${selectValue}`}
                            value={selectValue}
                            onValueChange={handleTypeChange}
                            disabled={disabled}
                        >
                            <SelectTrigger
                                className={cn(
                                    'h-9 shadow-none',
                                    !hasType && '[&>span]:text-sm [&>span]:text-muted-foreground',
                                    typeError && 'border-destructive focus-visible:ring-destructive',
                                )}
                            >
                                <SelectValue placeholder={typePlaceholder} />
                            </SelectTrigger>
                            <SelectContent className="max-h-[300px]">
                                {options.map((opt) => (
                                    <SelectItem key={opt.value || '__none__'} value={opt.value || '__none__'}>
                                        {opt.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                </div>
                {/* 输入行（仅在选中具体类型时显示） */}
                {hasType && (
                    <div className="flex items-center gap-3 w-full min-w-0">
                        <div className="flex-shrink-0 w-16" />
                        <div className="relative flex-1 min-w-0">
                            <ClearableInput
                                value={inputValue || ''}
                                placeholder={resolvedPlaceholder}
                                onChange={handleInputChange}
                                onClear={handleInputClear}
                                enableDoubleClickPaste={true}
                                showClearButton={true}
                                disabled={disabled}
                                className={cn(
                                    'h-9 text-sm shadow-none',
                                    valueError && 'border-destructive focus-visible:ring-destructive',
                                )}
                            />
                        </div>
                    </div>
                )}
                {error && (
                    <div className="text-xs text-destructive mt-1 pl-[76px]">
                        {error.message}
                    </div>
                )}
            </div>
        )
    }

    // ======================== 桌面端 ========================
    return (
        <div className={cn('flex flex-col min-w-[180px]', hasType && 'min-w-[340px]', className)}>
            <div className="text-sm font-medium text-muted-foreground mb-3">
                {label}
            </div>
            <div className="flex items-center gap-2">
                {/* 类型选择 */}
                <div className={cn(hasType ? 'w-[120px] flex-shrink-0' : 'flex-1')}>
                    <Select
                        key={`${String(typeName)}-${selectValue}`}
                        value={selectValue}
                        onValueChange={handleTypeChange}
                        disabled={disabled}
                    >
                        <SelectTrigger
                            className={cn(
                                'h-10 shadow-none',
                                !hasType && '[&>span]:text-sm [&>span]:text-muted-foreground',
                                typeError && 'border-destructive focus-visible:ring-destructive',
                            )}
                        >
                            <SelectValue placeholder={typePlaceholder} />
                        </SelectTrigger>
                        <SelectContent className="max-h-[300px]">
                            {options.map((opt) => (
                                <SelectItem key={opt.value || '__none__'} value={opt.value || '__none__'}>
                                    {opt.label}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                </div>
                {/* 输入框 */}
                {hasType && (
                    <div className="flex-1 min-w-0">
                        <ClearableInput
                            value={inputValue || ''}
                            placeholder={resolvedPlaceholder}
                            onChange={handleInputChange}
                            onClear={handleInputClear}
                            enableDoubleClickPaste={true}
                            showClearButton={true}
                            disabled={disabled}
                            className={cn(
                                'h-10 text-base shadow-none',
                                valueError && 'border-destructive focus-visible:ring-destructive',
                            )}
                        />
                    </div>
                )}
            </div>
            {error && (
                <div className="text-xs text-destructive mt-1">
                    {error.message}
                </div>
            )}
        </div>
    )
}
