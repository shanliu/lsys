import { useUserAppData, type UseUserAppDataOptions, type UserAppInfo } from "@apps/main/hooks/use-user-app-data"
import * as React from "react"
import { DynamicCombobox, type ComboboxOption } from "./dynamic-combobox"
import { Button } from "@shared/components/ui/button"
import { cn } from "@shared/lib/utils"

export interface UserAppSelectorProps {
  /** 当前选中的应用 ID */
  value?: number
  /** 应用选择变化时的回调 */
  onValueChange?: (appId: number | undefined, appInfo?: UserAppInfo) => void
  /** useAppData 的配置选项 */
  appDataOptions?: UseUserAppDataOptions
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
  /** 自定义选项渲染 */
  renderOption?: (app: UserAppInfo) => {
    label: string
    disabled?: boolean
  }
}

/**
 * 应用选择器组件
 * 基于 useAppData Hook 实现，支持分页加载和无限滚动
 */
export const UserAppSelector = React.forwardRef<HTMLButtonElement, UserAppSelectorProps>(
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
      renderOption,
    },
    ref
  ) => {
    // 当有指定 value 时，将其作为优先应用ID传递给 useAppData
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
    } = useUserAppData(finalAppDataOptions)

    // 转换应用数据为 ComboboxOption 格式
    const options: ComboboxOption[] = React.useMemo(() => {
      return apps.map((app) => {
        if (renderOption) {
          const customOption = renderOption(app)
          return {
            value: app.id.toString(),
            label: customOption.label,
            disabled: customOption.disabled,
          }
        }

        // 默认渲染逻辑
        let label = app.name

        if (showStatus) {
          const statusText = app.status === 1 ? "启用" : "禁用"
          label += ` (${statusText})`
        }

        if (showUserInfo && app.user_nickname) {
          label += ` - ${app.user_nickname}`
        }

        return {
          value: app.id.toString(),
          label,
          disabled: app.status !== 1, // 只有启用的应用才可选
        }
      })
    }, [apps, showStatus, showUserInfo, renderOption])

    // 处理值变化
    const handleValueChange = React.useCallback(
      (selectedValue: string) => {
        if (selectedValue === "") {
          onValueChange?.(undefined, undefined)
          return
        }

        const appId = parseInt(selectedValue, 10)
        const selectedApp = apps.find((app) => app.id === appId)

        if (selectedApp) {
          onValueChange?.(appId, selectedApp)
        }
      },
      [apps, onValueChange]
    )

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
        // 如果还有更多数据但数据较少，则预加载一页
        else if (hasMore && !isLoadingMore && apps.length < 20) {
          // 延迟加载，避免阻塞 UI
          setTimeout(() => {
            loadMore()
          }, 100)
        }
      }
    }, [isListLoaded, hasMore, apps.length, isLoadingMore, loadMore])

    // 获取当前选中的值（字符串格式）
    const currentValue = React.useMemo(() => {
      return value ? value.toString() : undefined
    }, [value])

    // 处理错误状态
    if (isError) {
      return (
        <DynamicCombobox
          ref={ref}
          options={[]}
          value=""
          disabled={true}
          placeholder={`加载失败: ${error || '未知错误'}`}
          triggerClassName={triggerClassName}
          maxHeight={maxHeight}
          emptyComponent={
            <div className="flex flex-col items-center justify-center py-6">
              <span className="text-sm text-muted-foreground mb-2">
                加载应用列表失败
              </span>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={refresh}
                className={cn("text-xs text-primary hover:underline p-0 h-auto")}
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
        triggerClassName={triggerClassName}
        maxHeight={maxHeight}
        enableInfiniteScroll={true}
        onScrollToBottom={handleScrollToBottom}
        hasMore={hasMore}
        loadingMore={isLoadingMore}
        emptyComponent={
          apps.length === 0 && !isLoading ? (
            <div className="flex flex-col items-center justify-center py-6">
              <span className="text-sm text-muted-foreground mb-2">
                暂无应用数据
              </span>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={refresh}
                className={cn("text-xs text-primary hover:underline p-0 h-auto")}
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

UserAppSelector.displayName = "UserAppSelector"

export type { UserAppInfo }

