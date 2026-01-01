import { DynamicCombobox, type ComboboxOption } from '@apps/main/components/selector/dynamic-combobox'
import { useSystemAppData, type SystemAppInfo, type UseSystemAppDataOptions } from '@apps/main/hooks/use-system-app-data'
import React from 'react'
import { Button } from '@shared/components/ui/button'
import { cn } from '@shared/lib/utils'

export interface SystemAppSelectorProps {
  /** 当前选中的应用 ID */
  value?: number
  /** 应用选择变化时的回调 */
  onValueChange?: (appId: number | undefined, appInfo?: SystemAppInfo) => void
  /** useSystemAppData 的配置选项 */
  appDataOptions?: UseSystemAppDataOptions
  /** 占位符文本 */
  placeholder?: string
  /** 是否禁用 */
  disabled?: boolean
  /** 触发器自定义样式类名（包含宽度、高度等样式） */
  triggerClassName?: string
  /** 最大高度 */
  maxHeight?: string
  /** 是否显示应用状态 */
  showStatus?: boolean
  /** 是否显示用户信息 */
  showUserInfo?: boolean
  /** 是否显示清除按钮 */
  showClearButton?: boolean
  /** 自定义选项渲染 */
  renderOption?: (app: SystemAppInfo) => {
    label: string
    disabled?: boolean
  }
  /** 布局参数，用于响应式高度 */
  layoutParams?: {
    isMobile?: boolean
  }
}

/**
 * 系统应用选择器组件
 * 基于 useSystemAppData Hook 实现，支持分页加载和无限滚动
 */
export const SystemAppSelector = React.forwardRef<HTMLButtonElement, SystemAppSelectorProps>(
  (
    {
      value,
      onValueChange,
      appDataOptions = {},
      placeholder = "选择应用...",
      disabled = false,
      triggerClassName = "w-[300px]",
      maxHeight = "max-h-[400px]",
      showStatus = true,
      showUserInfo = false,
      showClearButton = false,
      renderOption,
      layoutParams,
    },
    ref
  ) => {
    // 当有指定 value 时，将其作为优先应用ID传递给 useSystemAppData
    const finalAppDataOptions = React.useMemo(() => ({
      ...appDataOptions,
      priorityAppId: value || appDataOptions.priorityAppId,
    }), [value, appDataOptions])

    const {
      apps,
      isLoading,
      isError,
      error,
      hasMore,
      isLoadingMore,
      loadMore,
      refresh,
      isListLoaded,
      search,
      isSearching,
    } = useSystemAppData(finalAppDataOptions)

    // 转换应用数据为 ComboboxOption 格式
    const options: ComboboxOption[] = React.useMemo(() => {
      const appOptions = apps.map((app) => {
        if (renderOption) {
          const customOption = renderOption(app)
          return {
            value: app.id.toString(),
            ...customOption,
          }
        }

        // 默认渲染逻辑
        let label = app.name

        if (showStatus) {
          // 应用状态: 1=审核中, 2=正常, 3=被禁用
          const statusText = app.status === 2 ? '正常' : app.status === 1 ? '审核中' : '被禁用'
          label += ` (${statusText})`
        }

        if (showUserInfo && app.user_nickname) {
          label += ` - ${app.user_nickname}`
        }

        return {
          value: app.id.toString(),
          label,
          disabled: app.status !== 2, // 只有正常状态(status=2)的应用可选
        }
      })

      // 检查是否已经包含系统应用（id=0）
      const hasSystemApp = apps.some(app => app.id === 0)

      // 如果没有系统应用，则在选项开头添加一个系统选项
      if (!hasSystemApp) {
        const systemOption: ComboboxOption = {
          value: "0",
          label: "系统",
          disabled: false,
        }
        return [systemOption, ...appOptions]
      }

      return appOptions
    }, [apps, showStatus, showUserInfo, renderOption])

    // 处理值变化
    const handleValueChange = React.useCallback(
      (selectedValue: string) => {
        if (selectedValue === "") {
          onValueChange?.(undefined, undefined)
          return
        }

        const appId = parseInt(selectedValue, 10)
        
        // 处理系统应用（id=0）的特殊情况
        if (appId === 0) {
          onValueChange?.(0, undefined)
          return
        }

        const selectedApp = apps.find((app) => app.id === appId)

        if (selectedApp) {
          onValueChange?.(appId, selectedApp)
        }
      },
      [apps, onValueChange]
    )

    // 处理清除选择
    const handleClear = React.useCallback(() => {
      onValueChange?.(undefined, undefined)
    }, [onValueChange])

    // 处理搜索
    const handleSearchChange = React.useCallback((searchValue: string) => {
      if (searchValue.trim()) {
        // 使用 client_id 进行服务器端搜索
        search(searchValue.trim())
      } else {
        // 清空搜索，刷新原始列表
        search()
      }
    }, [search])

    // 处理滚动到底部，触发加载更多
    const handleScrollToBottom = React.useCallback(() => {
      if (hasMore && !isLoadingMore) {
        loadMore()
      }
    }, [hasMore, isLoadingMore, loadMore])

    // 当下拉框打开时，如果列表未加载，则触发加载
    const handlePopoverOpenChange = React.useCallback((open: boolean) => {
      if (open) {
        // 如果列表未加载，先加载列表
        if (!isListLoaded) {
          loadMore()
        }
        // 如果已加载但数据较少，尝试加载更多
        else if (hasMore && !isLoadingMore && apps.length < 20) {
          loadMore()
        }
      }
    }, [isListLoaded, hasMore, apps.length, isLoadingMore, loadMore])

    // 获取当前选中的值（字符串格式）
    const currentValue = React.useMemo(() => {
      return value !== undefined && value !== null ? value.toString() : undefined
    }, [value])

    // 计算触发器样式类名
    const finalTriggerClassName = React.useMemo(() => {
      const heightClass = layoutParams?.isMobile ? "h-9" : "h-10"
      return cn(triggerClassName, heightClass)
    }, [layoutParams?.isMobile, triggerClassName])

    // 处理错误状态
    if (isError) {
      return (
        <DynamicCombobox
          ref={ref}
          options={[]}
          value=""
          disabled={true}
          placeholder={`加载失败: ${error || '未知错误'}`}
          triggerClassName={finalTriggerClassName}
          maxHeight={maxHeight}
          emptyComponent={
            <div className="flex flex-col items-center justify-center py-6">
              <p className="text-sm text-muted-foreground">加载失败</p>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={refresh}
                className={cn("mt-2 text-xs text-primary hover:underline p-0 h-auto")}
              >
                点击重试
              </Button>
            </div>
          }
        />
      )
    }

    return (
      <DynamicCombobox
        ref={ref}
        options={options}
        value={currentValue}
        onValueChange={handleValueChange}
        onSearchChange={handleSearchChange}
        onOpenChange={handlePopoverOpenChange}
        placeholder={placeholder}
        searchPlaceholder="搜索应用..."
        noResultsText="未找到匹配的应用"
        loading={isLoading || isSearching}
        disabled={disabled}
        showClearButton={showClearButton}
        onClear={handleClear}
        triggerClassName={finalTriggerClassName}
        maxHeight={maxHeight}
        layoutParams={layoutParams}
        enableInfiniteScroll={true}
        onScrollToBottom={handleScrollToBottom}
        hasMore={hasMore}
        loadingMore={isLoadingMore}
        emptyComponent={
          apps.length === 0 && !isLoading ? (
            <div className="flex flex-col items-center justify-center py-6">
              <p className="text-sm text-muted-foreground">暂无应用数据</p>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={refresh}
                className={cn("mt-2 text-xs text-primary hover:underline p-0 h-auto")}
              >
                点击刷新
              </Button>
            </div>
          ) : undefined
        }
      />
    )
  }
)

SystemAppSelector.displayName = "SystemAppSelector"

export type { SystemAppInfo }
