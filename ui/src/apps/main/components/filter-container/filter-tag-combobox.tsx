import { LayoutParams } from '@apps/main/components/filter-container/container'
import { ComboboxOption, DynamicCombobox } from '@apps/main/components/selector/dynamic-combobox'
import { cn } from '@shared/lib/utils'
import React from 'react'
import { FieldPath, FieldValues, useController } from 'react-hook-form'

export interface FilterTagComboboxProps<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
> {
    name: TName
    placeholder?: string
    searchPlaceholder?: string
    noResultsText?: string
    className?: string
    label: string
    disabled?: boolean
    allLabel?: string
    layoutParams?: LayoutParams
    rules?: Parameters<typeof useController<TFieldValues, TName>>[0]['rules']
    /**
     * 获取标签名列表的异步函数
     * @param prefix 用户输入的前缀
     * @returns 标签名数组
     */
    fetchTagNames: (prefix: string) => Promise<string[]>
}

/**
 * 过滤器标签选择组件
 *
 * 基于 DynamicCombobox，支持输入前缀搜索标签。
 * 自动使用 react-hook-form 进行状态管理，支持移动端和桌面端响应式布局。
 * 可复用于 user / system 等不同场景，只需传入不同的 fetchTagNames。
 */
export function FilterTagCombobox<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
>({
    name,
    placeholder = '选择标签...',
    searchPlaceholder = '搜索标签...',
    noResultsText = '未找到标签',
    className,
    label,
    disabled = false,
    allLabel,
    layoutParams,
    rules,
    fetchTagNames,
}: FilterTagComboboxProps<TFieldValues, TName>) {
    const {
        field: { value, onChange },
        fieldState: { error }
    } = useController({
        name,
        rules,
    })

    const [options, setOptions] = React.useState<ComboboxOption[]>([])
    const [loading, setLoading] = React.useState(false)
    const abortRef = React.useRef<AbortController | null>(null)

    // 搜索回调（由 DynamicCombobox 的 onSearchChange 触发）
    const handleSearchChange = React.useCallback(
        async (search: string) => {
            // 取消上一次请求
            if (abortRef.current) {
                abortRef.current.abort()
            }
            const controller = new AbortController()
            abortRef.current = controller

            setLoading(true)
            try {
                const tags = await fetchTagNames(search)
                if (!controller.signal.aborted) {
                    const items: ComboboxOption[] = []
                    // 若有 allLabel 则添加到选项头部
                    if (allLabel) {
                        items.push({ value: '', label: allLabel })
                    }
                    tags.forEach(tag => {
                        items.push({ value: tag, label: tag })
                    })
                    setOptions(items)
                }
            } catch {
                if (!controller.signal.aborted) {
                    setOptions(allLabel ? [{ value: '', label: allLabel }] : [])
                }
            } finally {
                if (!controller.signal.aborted) {
                    setLoading(false)
                }
            }
        },
        [fetchTagNames, allLabel]
    )

    // 首次打开时加载
    const handleOpenChange = React.useCallback(
        (open: boolean) => {
            if (open && options.length === 0) {
                handleSearchChange('')
            }
        },
        [handleSearchChange, options.length]
    )

    const handleValueChange = React.useCallback(
        (val: string) => {
            onChange(val || undefined)
        },
        [onChange]
    )

    const handleClear = React.useCallback(() => {
        onChange(undefined)
    }, [onChange])

    // cleanup
    React.useEffect(() => {
        return () => {
            abortRef.current?.abort()
        }
    }, [])

    const combobox = (
        <DynamicCombobox
            options={options}
            value={value || ''}
            onValueChange={handleValueChange}
            onSearchChange={handleSearchChange}
            onOpenChange={handleOpenChange}
            placeholder={placeholder}
            searchPlaceholder={searchPlaceholder}
            noResultsText={noResultsText}
            loading={loading}
            disabled={disabled}
            showClearButton={!!value}
            onClear={handleClear}
            layoutParams={layoutParams}
            triggerClassName={cn(
                'shadow-none',
                layoutParams?.isMobile ? 'h-9 text-sm' : 'h-10 text-base'
            )}
        />
    )

    // 移动端
    if (layoutParams?.isMobile) {
        return (
            <div className={cn('flex items-center gap-3 w-full min-w-0', className)}>
                <div className="text-xs font-medium text-muted-foreground leading-none flex-shrink-0 w-16">
                    {label}
                </div>
                <div className="relative flex-1 min-w-0">
                    {combobox}
                    {error && (
                        <div className="text-xs text-destructive mt-1">
                            {error.message}
                        </div>
                    )}
                </div>
            </div>
        )
    }

    // 桌面端
    return (
        <div className={cn('flex flex-col min-w-[180px]', className)}>
            <div className="text-sm font-medium text-muted-foreground mb-3">
                {label}
            </div>
            {combobox}
            {error && (
                <div className="text-xs text-destructive mt-1">
                    {error.message}
                </div>
            )}
        </div>
    )
}
