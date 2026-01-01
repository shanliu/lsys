import { Button } from "@shared/components/ui/button"
import { useUserParentAppData, type UseUserParentAppDataOptions, type UserParentAppInfo } from "@apps/main/hooks/use-user-parent-app-data"
import { cn } from "@shared/lib/utils"
import { User as UserIcon } from "lucide-react"
import * as React from "react"
import { DynamicCombobox, type ComboboxOption } from "./dynamic-combobox"

export interface UserParentAppSelectorProps {
  /** 当前选中的父应用 ID */
  value?: string
  /** 父应用选择变化时的回调 */
  onValueChange?: (appId: string | undefined, appInfo?: UserParentAppInfo) => void
  /** useParentAppData 的配置选项 */
  appDataOptions?: Omit<UseUserParentAppDataOptions, 'keyword'>
  /** 占位符文本 */
  placeholder?: string
  /** 是否禁用 */
  disabled?: boolean
  /** 触发器自定义样式类名（包含宽度、高度等样式） */
  triggerClassName?: string
  /** 最大高度 */
  maxHeight?: string
  /** 是否显示用户信息 */
  showUserInfo?: boolean
  /** 自定义选项渲染 */
  renderOption?: (app: UserParentAppInfo) => Pick<ComboboxOption, "label" | "description" | "disabled" | "content">
  /** 布局参数，用于响应式样式 */
  layoutParams?: {
    isMobile?: boolean
  }
  /** 是否显示清除按钮 */
  showClearButton?: boolean
}

/**
 * 父应用选择器组件
 * 基于 useParentAppData Hook 实现，支持分页加载、无限滚动和搜索功能
 */
export const UserParentAppSelector = React.forwardRef<HTMLButtonElement, UserParentAppSelectorProps>(
  (
    {
      value,
      onValueChange,
      appDataOptions = {},
      placeholder = "选择父应用...",
      disabled = false,
      triggerClassName = "w-[300px]",
      maxHeight = "max-h-[400px]",
      showUserInfo = true,
      renderOption,
      layoutParams,
      showClearButton = true,
    },
    ref
  ) => {
    const [searchKeyword, setSearchKeyword] = React.useState<string>("")

    // 合并搜索关键词到 appDataOptions
    const finalAppDataOptions = React.useMemo(() => ({
      ...appDataOptions,
      keyword: searchKeyword || undefined,
    }), [searchKeyword, appDataOptions])

    const {
      apps,
      isLoading,
      isError,
      error,
      hasMore,
      isLoadingMore,
      isSearching,
      loadMore,
      refresh,
    } = useUserParentAppData(finalAppDataOptions)

    // 转换父应用数据为 ComboboxOption 格式
    const options: ComboboxOption[] = React.useMemo(() => {
      const appOptions = apps.map((app) => {
        if (renderOption) {
          const customOption = renderOption(app)
          return {
            value: String(app.id),
            label: customOption.label,
            description: customOption.description,
            disabled: customOption.disabled,
            content: customOption.content,
          }
        }

        // 默认渲染逻辑
        const label = `${app.name}${app.client_id ? ` (${app.client_id})` : ""}`
        const userNickname = app.user_nickname || "未绑定用户"
        const userAccount = app.user_account || "-"
        const userLine = showUserInfo ? `用户昵称: ${userNickname} (用户账号: ${userAccount})` : undefined

        return {
          value: String(app.id),
          label,
          description: userLine,
          disabled: false,
          content: (
            <div className="flex flex-col min-w-0">
              <div className="flex flex-wrap items-center gap-2 text-sm font-medium leading-tight text-foreground">
                <span>{app.name}</span>
                {app.client_id && (
                  <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
                    (
                    <span>{app.client_id}</span>
                    )
                  </span>
                )}
              </div>
              {showUserInfo && (
                <div className="mt-1 flex items-center gap-2 text-xs text-muted-foreground">
                  <UserIcon className="h-4 w-4" />
                  <span className="truncate">
                    {userNickname} ({userAccount})
                  </span>
                </div>
              )}
            </div>
          ),
        }
      })

      return appOptions
    }, [apps, showUserInfo, renderOption])

    // 处理值变化
    const handleValueChange = React.useCallback(
      (selectedValue: string) => {


        if (selectedValue === "") {
          onValueChange?.(undefined, undefined)
          return
        }

        const selectedApp = apps.find((app) => String(app.id) === selectedValue)


        if (selectedApp) {
          onValueChange?.(selectedValue, selectedApp)
        }
      },
      [apps, onValueChange]
    )

    // 处理清除
    const handleClear = React.useCallback(() => {
      onValueChange?.(undefined, undefined)
    }, [onValueChange])

    // 处理搜索变化 - 直接更新搜索关键词，延迟由 useDeferredValue 处理
    const handleSearchChange = React.useCallback(
      (search: string) => {
        setSearchKeyword(search)
      },
      []
    )

    // 处理滚动到底部，触发加载更多
    const handleScrollToBottom = React.useCallback(() => {
      if (hasMore && !isLoadingMore) {
        loadMore()
      }
    }, [hasMore, isLoadingMore, loadMore])

    // 当下拉框打开时，如果还有更多数据但数据较少，则预加载一页
    const handlePopoverOpenChange = React.useCallback((open: boolean) => {
      if (open && hasMore && !isLoadingMore && apps.length < 20) {
        // 延迟加载，避免阻塞 UI
        setTimeout(() => {
          loadMore()
        }, 100)
      }
    }, [hasMore, apps.length, isLoadingMore, loadMore])

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
                加载父应用列表失败
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
        value={value}
        onValueChange={handleValueChange}
        onSearchChange={handleSearchChange}
        onOpenChange={handlePopoverOpenChange}
        placeholder={placeholder}
        searchPlaceholder="搜索父应用..."
        noResultsText="未找到匹配的父应用"
        loading={isLoading}
        disabled={disabled}
        showClearButton={showClearButton}
        onClear={handleClear}
        triggerClassName={triggerClassName}
        maxHeight={maxHeight}
        layoutParams={layoutParams}
        enableInfiniteScroll={true}
        onScrollToBottom={handleScrollToBottom}
        hasMore={hasMore}
        loadingMore={isLoadingMore}
        emptyComponent={
          apps.length === 0 && !isLoading && !isSearching ? (
            <div className="flex flex-col items-center justify-center py-6">
              <span className="text-sm text-muted-foreground mb-2">
                {searchKeyword ? "未找到匹配的父应用" : "暂无父应用数据"}
              </span>
              {!searchKeyword && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={refresh}
                  className={cn("text-xs text-primary hover:underline p-0 h-auto")}
                >
                  点击刷新
                </Button>
              )}
            </div>
          ) : undefined
        }
      />
    )
  }
)

UserParentAppSelector.displayName = "UserParentAppSelector"

export type { UserParentAppInfo }

