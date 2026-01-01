import { Check, ChevronDown, Loader2, X } from "lucide-react"
import * as React from "react"

import { cn } from "@shared/lib/utils"

import { Button } from "@shared/components/ui/button"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@shared/components/ui/command"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@shared/components/ui/popover"

export interface ComboboxOption {
  value: string
  label: string
  description?: string
  disabled?: boolean
  content?: React.ReactNode
}

export interface ComboboxProps {
  /** 选项数组 */
  options: ComboboxOption[]
  /** 当前选中的值 */
  value?: string
  /** 值变化时的回调 */
  onValueChange?: (value: string) => void
  /** 搜索值变化时的回调 */
  onSearchChange?: (search: string) => void
  /** 下拉框打开状态变化时的回调 */
  onOpenChange?: (open: boolean) => void
  /** 搜索延迟时间（毫秒），默认为300ms */
  searchDelay?: number
  /** 是否使用 useDeferredValue 优化搜索性能，默认为true */
  useDeferredSearch?: boolean
  /** 占位符文本 */
  placeholder?: string
  /** 搜索占位符文本 */
  searchPlaceholder?: string
  /** 没有结果时显示的文本 */
  noResultsText?: string
  /** 是否正在加载 */
  loading?: boolean
  /** 是否禁用 */
  disabled?: boolean
  /** 是否显示清除按钮 */
  showClearButton?: boolean
  /** 清除时的回调 */
  onClear?: () => void
  /** 触发器自定义样式类名（包含宽度、高度等样式） */
  triggerClassName?: string
  /** 是否支持无限滚动 */
  enableInfiniteScroll?: boolean
  /** 滚动到底部时的回调 */
  onScrollToBottom?: () => void
  /** 是否还有更多数据 */
  hasMore?: boolean
  /** 是否正在加载更多 */
  loadingMore?: boolean
  /** 最大高度 */
  maxHeight?: string
  /** 自定义空状态 */
  emptyComponent?: React.ReactNode
  /** 自定义加载组件 */
  loadingComponent?: React.ReactNode
  /** 布局参数，用于响应式样式 */
  layoutParams?: {
    isMobile?: boolean
  }
}

export const DynamicCombobox = React.forwardRef<
  HTMLButtonElement,
  ComboboxProps
>(
  (
    {
      options = [],
      value,
      onValueChange,
      onSearchChange,
      onOpenChange,
      searchDelay = 300,
      useDeferredSearch = true,
      placeholder = "选择选项...",
      searchPlaceholder = "搜索选项...",
      noResultsText = "未找到匹配项",
      loading = false,
      disabled = false,
      showClearButton = false,
      onClear,
      triggerClassName,
      enableInfiniteScroll = false,
      onScrollToBottom,
      hasMore = false,
      loadingMore = false,
      maxHeight = "max-h-[400px]",
      emptyComponent,
      loadingComponent,
      layoutParams,
    },
    ref
  ) => {
    const [open, setOpen] = React.useState(false)
    const [searchValue, setSearchValue] = React.useState("")

    // 使用 useDeferredValue 来延迟搜索值的更新，优化性能
    const deferredSearchValue = React.useDeferredValue(searchValue)

    const listRef = React.useRef<HTMLDivElement>(null)
    const lastItemRef = React.useRef<HTMLDivElement>(null)
    const triggerRef = React.useRef<HTMLButtonElement>(null)
    const searchTimeoutRef = React.useRef<NodeJS.Timeout | undefined>(undefined)

    // 合并外部传入的 ref 和内部的 triggerRef
    React.useImperativeHandle(ref, () => triggerRef.current!, [])

    // 处理下拉框打开状态变化
    const handleOpenChange = React.useCallback(
      (newOpen: boolean) => {
        setOpen(newOpen)
        onOpenChange?.(newOpen)

        // 关闭时清理搜索状态
        if (!newOpen) {
          setSearchValue("")
          if (searchTimeoutRef.current) {
            clearTimeout(searchTimeoutRef.current)
          }
        }
      },
      [onOpenChange]
    )

    // 处理搜索变化 - 根据配置选择防抖方式
    const handleSearchChange = React.useCallback(
      (search: string) => {
        setSearchValue(search)

        if (useDeferredSearch) {
          // 使用 useDeferredValue，不需要额外的延迟
          return
        }

        // 使用 setTimeout 防抖
        if (searchTimeoutRef.current) {
          clearTimeout(searchTimeoutRef.current)
        }

        searchTimeoutRef.current = setTimeout(() => {
          onSearchChange?.(search)
        }, searchDelay)
      },
      [onSearchChange, searchDelay, useDeferredSearch]
    )

    // 当使用 useDeferredValue 时，监听 deferredSearchValue 变化并调用回调
    React.useEffect(() => {
      if (useDeferredSearch && onSearchChange) {
        onSearchChange(deferredSearchValue)
      }
    }, [deferredSearchValue, onSearchChange, useDeferredSearch])

    // 清理定时器
    React.useEffect(() => {
      return () => {
        if (searchTimeoutRef.current) {
          clearTimeout(searchTimeoutRef.current)
        }
      }
    }, [])

    // 处理值选择
    const handleSelect = React.useCallback(
      (selectedValue: string) => {
        if (selectedValue === value) {
          onValueChange?.("")
        } else {
          onValueChange?.(selectedValue)
        }
        setOpen(false)
      },
      [value, onValueChange]
    )

    // 使用 Intersection Observer 作为备用方案
    React.useEffect(() => {
      if (!enableInfiniteScroll || !onScrollToBottom || !lastItemRef.current || !hasMore || loadingMore) return

      const observer = new IntersectionObserver(
        (entries) => {
          const [entry] = entries
          if (entry.isIntersecting) {
            onScrollToBottom()
          }
        },
        {
          rootMargin: '20px',
        }
      )

      observer.observe(lastItemRef.current)
      return () => observer.disconnect()
    }, [enableInfiniteScroll, onScrollToBottom, hasMore, loadingMore, options.length])

    // 无限滚动处理
    React.useEffect(() => {
      if (!enableInfiniteScroll || !onScrollToBottom || !open) return

      // 确保 listRef 指向的是正确的滚动容器
      const scrollContainer = listRef.current
      if (!scrollContainer) return

      const handleScroll = () => {
        const { scrollTop, scrollHeight, clientHeight } = scrollContainer
        const isNearBottom = scrollTop + clientHeight >= scrollHeight - 20 // 增加触发区域

        if (isNearBottom && hasMore && !loadingMore) {
          onScrollToBottom()
        }
      }

      scrollContainer.addEventListener('scroll', handleScroll, { passive: true })
      return () => scrollContainer.removeEventListener('scroll', handleScroll)
    }, [enableInfiniteScroll, onScrollToBottom, hasMore, loadingMore, open])

    // 获取当前选中项的标签
    const selectedOption = React.useMemo(() => {
      const found = options.find((option) => option.value === value)
      return found
    }, [options, value])

    const displayValue = selectedOption?.label || placeholder
    const isShowingPlaceholder = !selectedOption

    return (
      <Popover open={open} onOpenChange={handleOpenChange}>
        <div className={cn("relative inline-flex w-full min-w-0", triggerClassName)}>
          <PopoverTrigger asChild>
            <Button
              ref={triggerRef}
              variant="outline"
              role="combobox"
              aria-expanded={open}
              className={cn(
                "w-full justify-between bg-transparent hover:bg-transparent hover:border-input font-normal text-foreground hover:text-foreground pr-2",
                "data-[state=open]:border-ring focus:border-ring focus:ring-1 focus:ring-ring",
                disabled && "cursor-not-allowed opacity-50",
                triggerClassName // 将 triggerClassName 也应用到 Button 上以继承高度等样式
              )}
              disabled={disabled}
            >
              <span className={cn(
                "truncate text-sm",
                isShowingPlaceholder && "text-muted-foreground"
              )}>{displayValue}</span>
              <div className="flex items-center gap-0.5 shrink-0 ml-2">
                {/* 清除按钮在箭头左边 */}
                {showClearButton && selectedOption && (
                  <span
                    role="button"
                    onClick={(e) => {
                      e.preventDefault()
                      e.stopPropagation()
                      onValueChange?.("")
                      onClear?.()
                    }}
                    className="hover:bg-muted rounded-sm transition-colors cursor-pointer inline-flex items-center justify-center p-0.5"
                    tabIndex={-1}
                  >
                    <X className={cn(
                      "text-muted-foreground hover:text-foreground",
                      layoutParams?.isMobile ? "h-3.5 w-3.5" : "h-4 w-4"
                    )} />
                  </span>
                )}
                {/* 下拉箭头在最右边 */}
                <ChevronDown className="h-4 w-4 shrink-0 opacity-50" />
              </div>
            </Button>
          </PopoverTrigger>
        </div>
        <PopoverContent
          className={cn("p-0 w-[var(--radix-popover-trigger-width)]")}
        >
          <Command shouldFilter={false}>
            <CommandInput
              placeholder={searchPlaceholder}
              value={searchValue}
              onValueChange={handleSearchChange}
            />
            <CommandList
              ref={listRef}
              className={cn(maxHeight, "overflow-y-auto")}
            >
              {loading && !loadingMore ? (
                <div className="flex items-center justify-center py-6">
                  {loadingComponent || (
                    <>
                      <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />
                      <span className="text-sm text-muted-foreground">加载中...</span>
                    </>
                  )}
                </div>
              ) : options.length === 0 ? (
                <CommandEmpty>
                  {emptyComponent || (
                    <span className="text-sm text-muted-foreground">
                      {noResultsText}
                    </span>
                  )}
                </CommandEmpty>
              ) : (
                <CommandGroup>
                  {options.map((option, index) => (
                    <CommandItem
                      key={option.value}
                      ref={index === options.length - 1 ? lastItemRef : undefined}
                      value={option.label}
                      disabled={option.disabled}
                      onSelect={() => handleSelect(option.value)}
                      className={cn("cursor-pointer")}
                    >
                      {option.content ? (
                        <div className="flex-1 min-w-0">{option.content}</div>
                      ) : (
                        <div className="flex flex-col flex-1 min-w-0">
                          <span className="truncate">{option.label}</span>
                          {option.description && (
                            <span className="truncate text-xs text-muted-foreground">{option.description}</span>
                          )}
                        </div>
                      )}
                      {value === option.value && (
                        <Check className={cn("ml-2 h-4 w-4 shrink-0 text-primary")} />
                      )}
                    </CommandItem>
                  ))}
                  {enableInfiniteScroll && loadingMore && (
                    <div className="flex items-center justify-center py-2">
                      <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />
                      <span className="text-xs text-muted-foreground">
                        加载更多...
                      </span>
                    </div>
                  )}
                </CommandGroup>
              )}
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    )
  }
)

DynamicCombobox.displayName = "DynamicCombobox"
