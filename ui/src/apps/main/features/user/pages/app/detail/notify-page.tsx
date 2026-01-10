"use client"

import { FilterContainer } from "@apps/main/components/filter-container/container"
import { FilterActions } from "@apps/main/components/filter-container/filter-actions"
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select"
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count"
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav"
import { NotifyKeyLink } from "@apps/main/features/user/components/ui/notify-key-link"
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data"
import { appQueryKey } from "@apps/main/lib/auth-utils"
import {
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  useCountNumManager,
  useOffsetPaginationHandlers,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils"
import { createStatusMapper } from "@apps/main/lib/status-utils"
import { Route } from "@apps/main/routes/_main/user/app/$appId/notify"
import { zodResolver } from "@hookform/resolvers/zod"
import {
  appList,
  appNotifyDel,
  appNotifyList,
  type AppListItemType,
  type AppNotifyListItemType,
} from "@shared/apis/user/app"
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog"
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error"
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table"
import { OffsetPagination } from "@shared/components/custom/pagination"
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table"
import { Badge } from "@shared/components/ui/badge"
import { Button } from "@shared/components/ui/button"
import { useToast } from "@shared/contexts/toast-context"
import { useIsMobile } from "@shared/hooks/use-mobile"
import {
  cn, extractMinMax, formatServerError, formatTime, getQueryResponseData,
  getQueryResponseNext,
  TIME_STYLE
} from "@shared/lib/utils"
import { DictList } from "@shared/types/apis-dict"
import { type LimitType } from "@shared/types/base-schema"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useNavigate } from "@tanstack/react-router"
import { type ColumnDef } from "@tanstack/react-table"
import { Eye, Key, Trash2 } from "lucide-react"
import React from "react"
import { notifyModuleConfig } from "./nav-info"
import { NotifyDetailDrawer } from "./notify-detail-drawer"
import {
  AppNotifyListFilterFormSchema
} from "./notify-schema"
import { NotifySecretDrawer } from "./notify-secret-drawer"

export function AppNotifyPage() {
  // docs\api\user\app\notify_del.md
  // docs\api\user\app\notify_list.md

  // docs\api\user\app\notify_secret_change.md
  // docs\api\user\app\app_secret_view.md[self]

  const { appId } = Route.useParams();

  // 获取应用详情
  const {
    data: appData,
    isLoading: isLoadingApp,
    isError: isAppError,
    error: appError
  } = useQuery({
    queryKey: appQueryKey(appId),
    queryFn: async () => {
      const result = await appList({
        app_id: Number(appId),
        page: { page: 1, limit: 1 },
        count_num: false
      })
      if (!result.status) {
        throw result
      }
      if (result.response?.data && result.response.data.length > 0) {
        return result.response.data[0]
      }
      return null
    },
    enabled: !!appId
  });
  const [secretDrawerOpen, setSecretDrawerOpen] = React.useState(false);
  const [detailDrawerOpen, setDetailDrawerOpen] = React.useState(false);
  const [selectedNotify, setSelectedNotify] = React.useState<AppNotifyListItemType | null>(null);

  // 字典数据获取 - 统一在最顶层获取一次
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(['user_app'] as const);

  // 字典数据已加载，创建状态映射器（需要在早期返回之前调用）
  const notifyStatusMapper = React.useMemo(
    () => dictData ? createStatusMapper<string>(
      {
        "1": "warning",   // 待回调 - 黄色警告色
        "2": "success",   // 已回调 - 绿色成功色
        "3": "danger",    // 回调失败 - 红色危险色
      },
      (status) => dictData.notify_status.getLabel(status) || status
    ) : null,
    [dictData]
  );

  // 如果字典加载失败，显示错误页面
  if (dictError && dictErrors.length > 0) {
    return (
      <CenteredError
        variant="page"
        error={dictErrors}
        onReset={refetchDict}
        className={cn("md:m-6")}
      />
    );
  }

  // 如果应用详情加载失败，显示错误页面
  if (isAppError) {
    return (
      <CenteredError
        variant="page"
        error={appError}
        className={cn("md:m-6")}
      />
    );
  }

  // 如果字典或应用详情加载中，显示骨架屏
  if (dictIsLoading || isLoadingApp) {
    return <PageSkeletonTable variant="page" className={cn("md:m-6")} />;
  }

  // 确保 dictData 和 notifyStatusMapper 已加载
  if (!dictData || !notifyStatusMapper) {
    return <PageSkeletonTable variant="page" className={cn("md:m-6")} />;
  }

  // 字典加载成功，渲染内容组件
  return (
    <>
      <AppDetailNavContainer
        {...notifyModuleConfig}
        actions={
          <Button variant="outline" size="sm" onClick={() => setSecretDrawerOpen(true)}>
            <Key className={cn("mr-2 h-4 w-4")} />
            回调密钥
          </Button>
        }
      >
        <AppNotifyContent
          dictData={dictData}
          notifyStatusMapper={notifyStatusMapper}
          appData={appData}
          onOpenDetail={(notify) => {
            setSelectedNotify(notify);
            setDetailDrawerOpen(true);
          }}
        />
      </AppDetailNavContainer>

      <NotifySecretDrawer
        appId={appId}
        open={secretDrawerOpen}
        onOpenChange={setSecretDrawerOpen}
      />

      <NotifyDetailDrawer
        notify={selectedNotify}
        isOpen={detailDrawerOpen}
        onOpenChange={setDetailDrawerOpen}
        dictData={dictData}
        notifyStatusMapper={notifyStatusMapper}
        appId={String(appId)}
      />
    </>
  )
}

// 内容组件：负责内容加载和渲染
interface AppNotifyContentProps {
  dictData: TypedDictData<["user_app"]>;
  notifyStatusMapper: ReturnType<typeof createStatusMapper<string>>;
  appData: AppListItemType | null | undefined;
  onOpenDetail: (notify: AppNotifyListItemType) => void;
}

function AppNotifyContent({ dictData, notifyStatusMapper, appData, onOpenDetail }: AppNotifyContentProps) {
  const { appId } = Route.useParams();
  const toast = useToast();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();

  // 从 URL 搜索参数获取过滤条件
  const filterParam = Route.useSearch();
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 过滤条件从 URL 参数获取
  const filters = {
    notify_method: filterParam.notify_method || null,
    notify_status: filterParam.notify_status || null,
  };

  // 分页状态 - 直接从 URL 参数派生
  const pagination: LimitType = {
    pos: filterParam.pos || null,
    limit: filterParam.limit || DEFAULT_PAGE_SIZE,
    forward: filterParam.forward || false,
    more: true,
    eq_pos: filterParam.eq_pos || false,
  };

  // 搜索导航函数
  const searchGo = useSearchNavigate(navigate, filterParam);

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  // 构建查询参数
  const queryParams = {
    app_id: Number(appId),
    count_num: countNumManager.getCountNum(),
    limit: pagination,
    attr_callback_data: true,
    method: filters.notify_method || undefined,
    status: filters.notify_status ? Number(filters.notify_status) : undefined,
  }

  // 获取回调通知列表
  const { data: notifyData, isSuccess, isLoading, isError, error } = useQuery({
    queryKey: [
      'app-notify-list',
      appId,
      pagination.pos,
      currentLimit,
      pagination.forward,
      pagination.more,
      pagination.eq_pos,
      filters.notify_method,
      filters.notify_status,
    ],
    queryFn: ({ signal }) => appNotifyList(queryParams, { signal }),
  });



  // 处理 Limit 分页查询结果（自动提取 total 和 next）
  isSuccess && countNumManager.handleLimitQueryResult(notifyData);

  // 获取响应数据
  const messages = getQueryResponseData<AppNotifyListItemType[]>(notifyData, []);
  const nextPageStartPos = getQueryResponseNext(notifyData);

  const deleteMutation = useMutation({
    mutationFn: (id: number) => appNotifyDel({ id }),
    onSuccess: () => {
      toast.success("回调记录删除成功")
      countNumManager.reset()
      queryClient.invalidateQueries({ queryKey: ['app-notify-list', appId] })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    }
  })

  const handleDelete = async (id: number) => {
    await deleteMutation.mutateAsync(id)
  }

  // 定义表格列
  const columns: ColumnDef<AppNotifyListItemType>[] = [
    {
      accessorKey: "id",
      header: () => <div className={cn(isMobile ? "" : "text-right")}>ID</div>,
      size: 60,
      cell: ({ row }) => <div className={cn(isMobile ? "" : "text-right")}>{row.original.id}</div>
    },
    {
      accessorKey: "notify_method",
      header: "方法",
      size: 80,
      cell: ({ row }) => (
        <span>
          {dictData.notify_method.getLabel(row.original.notify_method) || row.original.notify_method}
        </span>
      )
    },
    {
      accessorKey: "notify_key",
      header: "相关数据",
      size: 120,
      cell: ({ row }) => (
        <NotifyKeyLink
          appId={appId}
          notifyMethod={row.original.notify_method}
          notifyKey={row.original.notify_key}
          className={cn("bg-gray-100 bg-opacity-50 text-primary hover:underline")}
        />
      )
    },
    {
      accessorKey: "status",
      header: "状态",
      size: 80,
      cell: ({ row }) => (
        <Badge className={notifyStatusMapper.getClass(String(row.original.status))}>
          {notifyStatusMapper.getText(String(row.original.status))}
        </Badge>
      )
    },
    {
      accessorKey: "publish_time",
      header: "回调时间",
      size: 140,
      cell: ({ row }) => {
        const isSuccess = row.original.status === 2;
        const timeToShow = isSuccess ? row.original.publish_time : row.original.next_time;
        return <>{formatTime(timeToShow, TIME_STYLE.ABSOLUTE_ELEMENT)}</>;
      }
    },
    {
      id: "actions",
      size: 80,
      header: () => <div className="text-center">操作</div>,
      cell: ({ row }) => (
        <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
          <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
            <Button
              size="sm"
              variant="ghost"
              className={cn("h-7 px-2")}
              title="详情"
              onClick={() => onOpenDetail(row.original)}
            >
              <Eye className=" h-4 w-4" />
              <span className="ml-2">回调详情</span>
            </Button>
          </DataTableActionItem>
          <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
            <ConfirmDialog
              title="确认删除"
              description="确定要删除这条回调记录吗？"
              onConfirm={() => handleDelete(Number(row.original.id))}
            >
              <Button
                size="sm"
                variant="ghost"
                className={cn("h-7 px-2")}
                title="删除"
              >
                <Trash2 className=" h-4 w-4" />
                <span className="ml-2">{isMobile ? "删除" : "删除回调"}</span>
              </Button>
            </ConfirmDialog>
          </DataTableActionItem>
        </DataTableAction>
      )
    }
  ]


  // 使用统一的分页处理器
  const { handleNextPage, handlePrevPage, canGoNext, canGoPrev } =
    useOffsetPaginationHandlers({
      ...extractMinMax(messages, 'id', 'minId', 'maxId'),
      pagination,
      nextPageStartPos,
      searchGo,
      defaultForward: false, // 从大到小排序（新到旧）
    })

  // 刷新数据
  const handleRefresh = React.useCallback(() => {
    queryClient.refetchQueries({
      queryKey: ['app-notify-list'],
    })
  }, [queryClient])

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = React.useCallback(() => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ['app-notify-list'] });
  }, [queryClient, countNumManager])

  // 页面大小变化处理
  const handlePageSizeChange = React.useCallback(
    (pageSize: number) => {
      searchGo({
        limit: pageSize,
        pos: null, // 重置分页位置
        forward: false,
        eq_pos: false,
      })
    },
    [searchGo]
  )

  return (
    <div className="flex h-full flex-1 flex-col">
      {/* 过滤器 */}
      <div className="flex-shrink-0 mb-2">
        <FilterContainer
          defaultValues={{
            notify_method: filterParam.notify_method,
            notify_status: filterParam.notify_status,
          }}
          resolver={zodResolver(AppNotifyListFilterFormSchema) as any}
          onSubmit={(data) => {
            const transformedData = data as {
              notify_method?: string;
              notify_status?: string;
            };
            searchGo({
              notify_method: transformedData.notify_method,
              notify_status: transformedData.notify_status,
              pos: null,
              eq_pos: false,
            });
          }}
          onReset={() => {
            searchGo({
              pos: null,
              limit: currentLimit,
              forward: false,
              eq_pos: false,
              notify_method: undefined,
              notify_status: undefined,
            });
          }}
          countComponent={
            <FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />
          }
          className={cn("bg-card rounded-lg border shadow-sm relative")}
        >
          {(layoutParams, form) => (
            <div className={cn("flex-1 flex flex-wrap items-end gap-3")}>
              {/* 回调方法过滤 */}
              {dictData.notify_method && (() => {
                // 如果有父应用，过滤掉 sub_app_notify
                let notifyMethodData = dictData.notify_method;
                if (appData?.parent_app_id && appData.parent_app_id > 0) {
                  const filtered = dictData.notify_method.filter((item: any) => item.key !== 'sub_app_notify');
                  notifyMethodData = new DictList(...filtered);
                }

                return (
                  <FilterDictSelect
                    name="notify_method"
                    placeholder="选择回调方法"
                    label="回调类型"
                    disabled={isLoading}
                    dictData={notifyMethodData}
                    layoutParams={layoutParams}
                    allLabel="全部"
                  />
                );
              })()}

              {/* 回调状态过滤 */}
              {dictData.notify_status && (
                <FilterDictSelect
                  name="notify_status"
                  placeholder="选择状态"
                  label="状态"
                  disabled={isLoading}
                  dictData={dictData.notify_status}
                  layoutParams={layoutParams}
                  allLabel="全部"
                />
              )}

              {/* 动作按钮区域 */}
              <div className={cn(layoutParams.isMobile ? "w-full" : "flex-shrink-0")}>
                <FilterActions
                  form={form}
                  loading={isLoading}
                  layoutParams={layoutParams}
                  onRefreshSearch={clearCacheAndReload}
                />
              </div>
            </div>
          )}
        </FilterContainer>
      </div>

      <div className="flex flex-col flex-1 min-h-0">
        <DataTable
          data={messages}
          columns={columns}
          loading={isLoading}
          error={isError ? <CenteredError error={error} variant="content" onReset={handleRefresh} /> : null}

          className={cn("flex-1")}
        />
        <div className={cn("py-4")}>
          {(countNumManager.getTotal() ?? 0) > 0 && (
            <OffsetPagination
              limit={currentLimit}
              hasNext={canGoNext}
              canGoPrev={canGoPrev}
              total={countNumManager.getTotal()}
              currentPageSize={messages.length}
              loading={isLoading}
              onNext={handleNextPage}
              onPrevious={handlePrevPage}
              onRefresh={handleRefresh}
              showRefresh={true}
              showPageSize={true}
              pageSizeOptions={PAGE_SIZE_OPTIONS}
              onPageSizeChange={handlePageSizeChange}
            />
          )}
        </div>
      </div>
    </div>
  )
}