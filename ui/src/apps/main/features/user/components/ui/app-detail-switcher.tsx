import { Avatar, AvatarFallback } from '@shared/components/ui/avatar'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@shared/components/ui/command'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@shared/components/ui/popover'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { useIsMobile } from '@shared/hooks/use-mobile'
import { useDictData, type TypedDictData } from '@apps/main/hooks/use-dict-data'
import { useUserAppData, type UserAppInfo } from '@apps/main/hooks/use-user-app-data'
import { cn } from '@shared/lib/utils'
import { createStatusMapper } from '@apps/main/lib/status-utils'
import { useNavigate } from '@tanstack/react-router'
import { Building2, CheckCircle, ChevronsUpDown, Clock,  User, XCircle } from 'lucide-react'
import * as React from 'react'

interface AppDetailSwitcherProps {
  userAppId: number
}

export function AppDetailSwitcher({ userAppId }: AppDetailSwitcherProps) {
  // 字典数据获取 - 统一在最顶层获取一次
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["user_app"] as const);

  // 如果字典加载失败，显示错误页面
  if (dictError && dictErrors.length > 0) {
    return (
      <CenteredError
        variant="content"
        error={dictErrors}
        onReset={refetchDict}
        className={cn("p-2")}
      />
    );
  }

  // 如果字典加载中，显示加载状态
  if (dictIsLoading) {
    return (
      <CenteredLoading variant="content" iconSize="xs" className={cn("py-3")} />
    );
  }

  // 字典加载成功，渲染内容组件
  return (
    <AppDetailSwitcherContent userAppId={userAppId} dictData={dictData} />
  );
}

// 内容组件：负责内容加载和渲染
interface AppDetailSwitcherContentProps {
  userAppId: number;
  dictData: TypedDictData<["user_app"]>;
}

function AppDetailSwitcherContent({ userAppId, dictData }: AppDetailSwitcherContentProps) {
  const navigate = useNavigate()
  const isMobile = useIsMobile()
  const currentAppId = userAppId
  const [open, setOpen] = React.useState(false)
  const [searchValue, setSearchValue] = React.useState("")

  // 使用 useDeferredValue 来延迟搜索值更新
  const deferredSearchValue = React.useDeferredValue(searchValue)

  // 使用 useAppData Hook 获取应用数据 - 支持懒加载和搜索
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
  } = useUserAppData({
    priorityAppId: currentAppId, // 优先加载当前应用信息
    pageSize: 20, // 每页加载20个应用
    autoLoad: true, // 自动加载优先应用
    client_id: deferredSearchValue.trim() || undefined, // 使用延迟后的搜索值作为 client_id 搜索
  })

  // 根据是否在搜索来决定显示状态
  const isSearching = !!deferredSearchValue.trim()

  // 根据当前URL参数确定活跃应用
  const activeApp = React.useMemo(() => {
    // 如果有当前应用ID，尝试从应用列表中找到
    if (currentAppId) {
      const foundApp = apps.find(app => app.id === currentAppId)
      return foundApp || null
    }

    // 如果没有指定 currentAppId，返回第一个应用
    return apps[0] || null
  }, [apps, currentAppId])

  // 处理搜索 - 简化为只设置搜索值，React Query 会自动根据 client_id 参数变化重新获取数据
  const handleSearch = React.useCallback((value: string) => {
    setSearchValue(value)
  }, [])

  // 检查当前选中的应用是否应该被禁用
  // 当在搜索状态下，搜索结果为空，且搜索字符与当前应用的client_id不匹配时，禁用当前应用
  const isActiveAppDisabled = React.useMemo(() => {
    if (!isSearching || !activeApp || !deferredSearchValue.trim()) {
      return false
    }

    // 检查搜索字符是否与当前应用的client_id匹配
    const searchMatches = activeApp.client_id.toLowerCase().includes(deferredSearchValue.trim().toLowerCase())

    // 如果搜索不匹配当前应用，则禁用
    // 注意：由于useAppData的实现，当有priorityAppId时，apps数组总是包含当前应用
    // 所以我们不能简单地检查apps.length === 0，而要检查搜索是否匹配
    return !searchMatches
  }, [isSearching, activeApp, deferredSearchValue])

  // 当应用数据加载完成且没有选中应用时，自动选择第一个应用 - 此逻辑现在不需要，因为 currentAppId 总是存在
  // React.useEffect(() => {
  //   if (!isSearching && apps.length > 0 && !currentAppId && apps[0]) {
  //     navigate({
  //       to: '/user/app/$appId',
  //       params: { appId: apps[0].id },
  //       replace: true,
  //     })
  //   }
  // }, [apps, currentAppId, navigate, isSearching])

  // 当有 currentAppId 但没有找到对应应用时，触发完整列表加载
  React.useEffect(() => {
    if (currentAppId && !isSearching && !isLoading && apps.length > 0) {
      const foundApp = apps.find(app => app.id === currentAppId)
      if (!foundApp && !isListLoaded) {
        // 如果设置了优先应用但在当前列表中找不到，加载完整列表
        loadMore()
      }
    }
  }, [currentAppId, apps, isSearching, isLoading, isListLoaded, loadMore])

  // 处理应用切换
  const handleAppSwitch = React.useCallback((app: UserAppInfo) => {
    navigate({
      to: '/user/app/$appId',
      params: { appId: app.id },
    })
    setOpen(false)
    setSearchValue("")
  }, [navigate])

  // 处理弹出框打开 - 懒加载触发
  const handleOpenChange = React.useCallback((newOpen: boolean) => {
    setOpen(newOpen)
    if (newOpen && !isListLoaded && !isSearching) {
      // 懒加载：打开时触发列表加载
      loadMore()
    }
    if (!newOpen) {
      setSearchValue("")
    }
  }, [isListLoaded, loadMore, isSearching])

  // 处理滚动加载更多
  const handleScroll = React.useCallback((e: React.UIEvent<HTMLDivElement>) => {
    const element = e.currentTarget
    const { scrollTop, scrollHeight, clientHeight } = element

    // 当滚动到底部附近时加载更多
    if (scrollHeight - scrollTop <= clientHeight + 100 && hasMore && !isLoadingMore) {
      loadMore()
    }
  }, [hasMore, isLoadingMore, loadMore])

  // 创建应用状态映射器
  const appStatusMapper = React.useMemo(() => 
    createStatusMapper(
      {
        2: "success",   // 正常 - 绿色成功色
        1: "neutral",   // 审核中 - 灰色次要色
        3: "danger",    // 被禁用 - 红色危险色
      },
      (status) =>
        dictData.app_status.getLabel(String(status)) || String(status)
    ),
    [dictData.app_status]
  );

  // 获取状态信息
  const getStatusInfo = (status: number) => {
    const statusText = appStatusMapper.getText(status);
    const statusClass = appStatusMapper.getClass(status);
    
    // 根据状态选择对应的图标
    let icon: React.ElementType = Clock;
    switch (status) {
      case 2:
        icon = CheckCircle;
        break;
      case 1:
        icon = Clock;
        break;
      case 3:
        icon = XCircle;
        break;
      default:
        icon = Clock;
    }

    return {
      icon,
      text: statusText,
      variant: "outline" as const,
      class: statusClass,
    }
  }

  // 如果出错，显示错误状态
  if (isError) {
    return (
      <div className="flex items-center space-x-2">
        <div className="text-sm text-red-600 flex items-center">
          <XCircle className={cn("h-4 w-4 mr-1")} />
          {error || "加载应用失败"}
        </div>
        <Button variant="outline" size="sm" onClick={refresh}>
          重试加载
        </Button>
      </div>
    )
  }

  // 如果没有应用数据且不在加载中，显示空状态
  // 注意：搜索状态下不在此处显示，而是在下拉框内显示"未找到匹配的应用"
  if (!isLoading && apps.length === 0 && !isError && isListLoaded && !isSearching) {
    return (
      <div className="flex items-center space-x-2">
        <div className="text-sm text-muted-foreground flex items-center">
          <Building2 className={cn("h-4 w-4 mr-1")} />
          无相关应用
        </div>
      </div>
    )
  }

  // 如果正在加载且没有任何应用数据（包括优先应用），且不在搜索状态，显示加载状态
  // 注意：如果已经有 activeApp（优先应用）或正在搜索，则显示切换框，在下拉列表内显示加载状态
  if (isLoading && apps.length === 0 && !activeApp && !isSearching) {
    return (
      <CenteredLoading variant="content" iconSize="xs" className={cn("py-3")} />
    )
  }


  return (
    <Popover open={open} onOpenChange={handleOpenChange}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className={cn(
            isMobile ? "px-2 max-w-[calc(100vw-8rem)]" : "px-1",
            isActiveAppDisabled && "opacity-50"
          )}
        >
          {activeApp ? (
            <div className="flex items-center gap-2 flex-1 min-w-0">
              <Avatar className={cn(isMobile ? "p-1 h-7 w-7 flex-shrink-0" : "p-2", isActiveAppDisabled && "opacity-50")}>
                <AvatarFallback className={cn(
                  "text-xs font-bold",
                  isActiveAppDisabled
                    ? "bg-muted text-muted-foreground"
                    : "bg-primary text-primary-foreground"
                )}>
                  {activeApp.name.substring(0, 1).toUpperCase()}
                </AvatarFallback>
              </Avatar>
              <span className={`truncate text-sm ${isActiveAppDisabled ? "text-muted-foreground" : ""}`}>
                {activeApp.name}
              </span>
            </div>
          ) : (
            <span className="text-muted-foreground">选择应用...</span>
          )}
          <ChevronsUpDown className={cn("ml-2 h-4 w-4 shrink-0 opacity-50")} />
        </Button>
      </PopoverTrigger>
      <PopoverContent
        className={cn("p-0", isMobile ? "w-[90vw] mr-[5vw]" : "w-[400px]")}
        align={isMobile ? "start" : "end"}
      >
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="搜索应用..."
            value={searchValue}
            onValueChange={handleSearch}
            className={cn("border-b")}
          />
          <CommandList className={cn("max-h-[300px]")} onScroll={handleScroll}>
            {!isListLoaded && isLoading ? (
              <CenteredLoading variant="content" iconSize="xs" className={cn("py-6.5")}/>
            ) : isSearching && isLoading ? (
              <CenteredLoading variant="content" iconSize="xs" className={cn("py-6.5")} />
            ) : isSearching && isActiveAppDisabled && apps.length === 1 && activeApp ? (
              <CommandGroup>
                {/* 当搜索时只有当前应用且不匹配时，显示禁用的当前应用 */}
                <CommandItem
                  key={activeApp.id}
                  value={activeApp.id.toString()}
                  disabled
                  className={cn("flex items-center space-x-3 px-3 py-3 opacity-50 cursor-not-allowed")}
                >
                  <Avatar className={cn("h-8 w-8")}>
                    <AvatarFallback className={cn("text-xs font-bold bg-muted text-muted-foreground")}  >
                      {activeApp.name.substring(0, 1).toUpperCase()}
                    </AvatarFallback>
                  </Avatar>

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center space-x-2 mb-1">
                      <span className="font-medium text-sm truncate text-muted-foreground">{activeApp.name}</span>
                      <Badge variant="outline" className={cn("text-xs")}>
                        当前应用
                      </Badge>
                    </div>

                    <div className="flex items-center space-x-4 text-xs text-muted-foreground">
                      <div className="flex items-center space-x-1">
                        <Building2 className={cn("h-3 w-3")} />
                        <span className="truncate">ID: {activeApp.client_id}</span>
                      </div>

                      {activeApp.user_nickname && (
                        <div className="flex items-center space-x-1">
                          <User className={cn("h-3 w-3")} />
                          <span className="truncate">{activeApp.user_nickname}</span>
                        </div>
                      )}
                    </div>
                  </div>

                  <CheckCircle className={cn("h-4 w-4 text-muted-foreground")} />
                </CommandItem>
                <CommandEmpty>
                  未找到匹配的应用
                </CommandEmpty>
              </CommandGroup>
            ) : apps.length === 0 ? (
              <CommandEmpty>
                {searchValue ? "未找到匹配的应用" : "暂无应用数据"}
              </CommandEmpty>
            ) : (
              <CommandGroup>
                {apps.map((app) => {
                  const statusInfo = getStatusInfo(app.status)
                  const StatusIcon = statusInfo.icon

                  return (
                    <CommandItem
                      key={app.id}
                      value={app.id.toString()}
                      onSelect={() => handleAppSwitch(app)}
                      className={cn("flex items-center space-x-3 px-3 py-3 cursor-pointer")}
                    >
                      <Avatar className={cn("h-8 w-8")}>
                        <AvatarFallback className={cn("text-xs font-bold bg-primary text-primary-foreground")}>
                          {app.name.substring(0, 1).toUpperCase()}
                        </AvatarFallback>
                      </Avatar>

                      <div className="flex-1 min-w-0">
                        <div className="flex items-center space-x-2 mb-1">
                          <span className="font-medium text-sm truncate">{app.name}</span>
                          <Badge variant={statusInfo.variant} className={cn("text-xs", statusInfo.class)}>
                            <StatusIcon className={cn("h-3 w-3 mr-1")} />
                            {statusInfo.text}
                          </Badge>
                        </div>

                        <div className="flex items-center space-x-4 text-xs text-muted-foreground">
                          <div className="flex items-center space-x-1">
                            <Building2 className={cn("h-3 w-3")} />
                            <span className="truncate">ID: {app.client_id}</span>
                          </div>

                          {app.user_nickname && (
                            <div className="flex items-center space-x-1">
                              <User className={cn("h-3 w-3")} />
                              <span className="truncate">{app.user_nickname}</span>
                            </div>
                          )}
                        </div>
                      </div>

                      {currentAppId === app.id && (
                        <CheckCircle className={cn("h-4 w-4 text-primary")} />
                      )}
                    </CommandItem>
                  )
                })}

                {/* 加载更多指示器 */}
                {isLoadingMore && (
                  <CenteredLoading variant="content" iconSize="xs" className={cn("py-6.5")} />
                )}
              </CommandGroup>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}
