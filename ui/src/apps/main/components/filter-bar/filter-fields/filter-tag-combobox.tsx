import { FilterFieldContext, LayoutParams } from '../context'
import { ComboboxOption, DynamicCombobox } from '@apps/main/components/selector/dynamic-combobox'
import { cn } from '@shared/lib/utils'
import React from 'react'
import { FieldPath, FieldValues, useController } from 'react-hook-form'

/** 标签选项类型：支持纯字符串或 {value, label} 对象，适配不同接口返回 */
export type TagItem = string | { value: string; label: string }

export interface FilterTagComboboxProps<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>
> {
    name: TName
    placeholder?: string
    searchPlaceholder?: string
    noResultsText?: string
    fetchErrorText?: string
    className?: string
    label: string
    disabled?: boolean
    allLabel?: string
    layoutParams?: LayoutParams
    rules?: Parameters<typeof useController<TFieldValues, TName>>[0]['rules']
    /**
     * 获取标签列表的异步函数
     * @param prefix 用户输入的前缀
     * @param signal AbortSignal，可传入请求配置实现取消
     * @returns 标签项数组（字符串或 {value, label} 对象）
     */
    fetchTagNames: (prefix: string, signal: AbortSignal) => Promise<TagItem[]>
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
    fetchErrorText = '加载标签失败',
    className,
    label,
    disabled = false,
    allLabel,
    layoutParams,
    rules,
    fetchTagNames,
}: FilterTagComboboxProps<TFieldValues, TName>) {
    const filterCtx = React.useContext(FilterFieldContext);
    const effectiveLayoutParams = layoutParams ?? filterCtx?.layoutParams ?? { isMobile: false };
    const {
        field: { value, onChange },
        fieldState: { error }
    } = useController({
        name,
        rules,
    })

    const [options, setOptions] = React.useState<ComboboxOption[]>([])
    const [loading, setLoading] = React.useState(false)
    const [fetchError, setFetchError] = React.useState<string | null>(null)
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
            setFetchError(null)
            try {
                const tags = await fetchTagNames(search, controller.signal)
                if (!controller.signal.aborted) {
                    const items: ComboboxOption[] = []
                    // 若有 allLabel 则添加到选项头部
                    if (allLabel) {
                        items.push({ value: '', label: allLabel })
                    }
                    tags.forEach(tag => {
                        if (typeof tag === 'string') {
                            items.push({ value: tag, label: tag })
                        } else {
                            items.push(tag)
                        }
                    })
                    setOptions(items)
                }
            } catch (err) {
                if (!controller.signal.aborted) {
                    // Suppress abort/cancel errors (e.g. from react-query dedup in StrictMode)
                    const isAbortError =
                        err != null &&
                        typeof err === 'object' &&
                        ((err as any).code === 'ERR_CANCELED' ||
                            (err as any).name === 'AbortError' ||
                            (err as any).name === 'CanceledError')
                    if (!isAbortError) {
                        setFetchError(fetchErrorText)
                        setOptions([])
                    }
                }
            } finally {
                if (!controller.signal.aborted) {
                    setLoading(false)
                }
            }
        },
        [fetchTagNames, allLabel, fetchErrorText]
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

    const combobox = fetchError ? (
        <DynamicCombobox
            options={[]}
            value=""
            disabled={true}
            placeholder={fetchErrorText}
            layoutParams={effectiveLayoutParams}
            triggerClassName={cn(
                'shadow-none',
                effectiveLayoutParams.isMobile ? 'h-9 text-sm' : 'h-10 text-base'
            )}
            emptyComponent={
                <div className="flex flex-col items-center justify-center py-4 gap-2">
                    <span className="text-xs text-muted-foreground">{fetchErrorText}</span>
                    <button
                        type="button"
                        className="text-xs text-primary hover:underline"
                        onClick={() => handleSearchChange('')}
                    >
                        点击重试
                    </button>
                </div>
            }
        />
    ) : (
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
            layoutParams={effectiveLayoutParams}
            triggerClassName={cn(
                'shadow-none',
                effectiveLayoutParams.isMobile ? 'h-9 text-sm' : 'h-10 text-base'
            )}
        />
    )

    // 移动端
    if (effectiveLayoutParams.isMobile) {
        return (
            <div className="flex items-center gap-3 w-full min-w-0">
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
