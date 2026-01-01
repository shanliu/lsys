import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { UserDataTooltip } from "@apps/main/components/local/user-data-tooltip";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import { DEFAULT_PAGE_SIZE, useCountNumManager } from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/admin/app/list";
import { zodResolver } from "@hookform/resolvers/zod";
import { appList, type AppItemType } from "@shared/apis/admin/app";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { PagePagination } from "@shared/components/custom/pagination";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useIsMobile } from "@shared/hooks/use-mobile";
import {
  cn,
  formatTime,
  getQueryResponseData,
  TIME_STYLE,
} from "@shared/lib/utils";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ColumnDef } from "@tanstack/react-table";
import { Eye, List } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { AppDetailDrawer } from "./list-detail-drawer";
import {
  AdminAppListFilterFormSchema
} from "./list-schema";
import { ListSubAppDrawer } from "./list-sub-app-drawer";

export function AppListPage() {
  // 字典数据获取 - 统一在最顶层获取一次
  // system\app\mapping.md
  // system\app\list.md
  // system\app\sub_list.md
  // system\app\app_logout.md
  // system\app\delete.md
  // system\app\disable.md
  // system\app\oauth_clear_access_token.md
  // system\app\oauth_clear_refresh_token.md
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["admin_app"] as const);

  // 如果字典加载失败，显示错误页面
  if (dictError && dictErrors.length > 0) {
    return (
      <CenteredError
        variant="page"
        error={dictErrors}
        onReset={refetchDict}
        className={cn("m-4 md:m-6")}
      />
    );
  }

  // 如果字典加载中，显示骨架屏
  if (dictIsLoading) {
    return <PageSkeletonTable variant="page" className={cn('m-4 md:m-6')} />;
  }

  // 字典加载成功，渲染内容组件
  return <AppListContent dictData={dictData} />;
}

// 内容组件：负责内容加载和渲染
interface AppListContentProps {
  dictData: TypedDictData<["admin_app"]>;
}

function AppListContent({ dictData }: AppListContentProps) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();

  // 获取 URL search 参数
  const filterParam = Route.useSearch();

  // 应用详情抽屉状态
  const [detailDrawer, setDetailDrawer] = useState({
    open: false,
    appId: null as number | null,
  });

  // 子应用列表抽屉状态
  const [subAppDrawer, setSubAppDrawer] = useState({
    open: false,
    appId: null as number | null,
  });

  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 过滤条件
  const filters = {
    app_name: filterParam.app_name || null,
    status: filterParam.status ?? null,
    user_id: filterParam.user_id ?? null,
    client_id: filterParam.client_id || null,
    app_id: filterParam.app_id ?? null,
  };

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  // 获取应用列表数据
  const { data: appData, isSuccess, isLoading, isError, error } = useQuery({
    queryKey: [
      "appList",
      filterParam.page || 1,
      currentLimit,
      filterParam.app_name,
      filterParam.status,
      filterParam.user_id,
      filterParam.client_id,
      filterParam.app_id,
    ],
    queryFn: ({ signal }) =>
      appList(
        {
          page: {
            page: filterParam.page || 1,
            limit: currentLimit,
          },
          count_num: countNumManager.getCountNum(),
          detail_data: false, // 列表页面不需要详细数据
          ...(filterParam.app_name && { app_name: filterParam.app_name }),
          ...(filterParam.status !== undefined && { status: filterParam.status }),
          ...(filterParam.user_id !== undefined && { user_id: filterParam.user_id }),
          ...(filterParam.client_id && { client_id: filterParam.client_id }),
          ...(filterParam.app_id !== undefined && { app_id: filterParam.app_id }),
        },
        { signal },
      ),
    placeholderData: (previousData) => previousData,
  });

  // 处理 Page 分页查询结果（自动提取 total）
  isSuccess && countNumManager.handlePageQueryResult(appData);

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["appList"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["appList"] });
  };

  // 处理分页变化
  const handlePageChange = (page: number) => {
    navigate({
      search: {
        ...filterParam,
        page,
      } as any,
    });
  };

  // 处理查看详情（用 useCallback 因为在 columns useMemo 中使用）
  const handleViewDetail = useMemo(
    () => (appId: number) => {
      setDetailDrawer({ open: true, appId });
    },
    [],
  );

  // 处理查看子应用列表（用 useCallback 因为在 columns useMemo 中使用）
  const handleViewSubApps = useMemo(
    () => (appId: number) => {
      setSubAppDrawer({ open: true, appId });
    },
    [],
  );

  // 处理查看待审核请求（用 useCallback 因为在 columns useMemo 中使用）
  const handleViewRequests = useMemo(
    () => (appId: number) => {
      navigate({
        to: "/admin/app/request",
        search: { app_id: appId },
      });
    },
    [navigate],
  );

  // 关闭详情抽屉
  const handleCloseDetailDrawer = () => {
    setDetailDrawer({ open: false, appId: null });
  };

  // 关闭子应用列表抽屉
  const handleCloseSubAppDrawer = () => {
    setSubAppDrawer({ open: false, appId: null });
  };

  const apps = getQueryResponseData<AppItemType[]>(appData, []);

  // 自动打开详情抽屉（当 detail=1 且 app_id>0 且数据加载完成时）
  useEffect(() => {
    if (
      filterParam.detail === 1 &&
      filterParam.app_id &&
      filterParam.app_id > 0 &&
      !isLoading &&
      apps.length > 0
    ) {
      // 查找该 app_id 对应的应用对象
      const targetApp = apps.find(app => app.id === filterParam.app_id);
      if (targetApp) {
        setDetailDrawer({ open: true, appId: targetApp.id });
      }
    }
  }, [filterParam.detail, filterParam.app_id, isLoading, apps]);

  // 状态样式映射 - 应用状态: 1=审核中, 2=正常, 3=被禁用
  const statusMapper = useMemo(
    () =>
      createStatusMapper(
        { 1: "warning", 2: "success", 3: "neutral" },
        (status: number) => {
          return dictData.app_status?.getLabel(String(status)) || String(status);
        },
      ),
    [dictData.app_status],
  );

  // 定义表格列
  const columns = useMemo<ColumnDef<AppItemType>[]>(
    () => [
      {
        accessorKey: "id",
        header: () => <div className={cn(isMobile ? "" : "text-right")}>ID</div>,
        size: 80,
        cell: ({ getValue }) => (
          <div className={cn("font-mono text-xs", isMobile ? "" : "text-right")}>{getValue<number>()}</div>
        ),
      },
      {
        accessorKey: "name",
        header: "应用名称",
        cell: ({ getValue }) => {
          const name = getValue<string>();
          return <div className="font-medium text-foreground">{name}</div>;
        },
      },
      {
        accessorKey: "client_id",
        header: "应用标识",
        cell: ({ getValue }) => {
          const clientId = getValue<string | null>();
          return clientId ? (
            <CopyableText
              value={clientId}
              message="应用标识已复制"
              maxWidthClassName="max-w-[200px]"
            />
          ) : (
            <span className="text-muted-foreground">-</span>
          );
        },
      },
      {
        accessorKey: "status",
        header: "状态",
        cell: ({ getValue }) => {
          const status = getValue<number>();
          return (
            <Badge variant="secondary" className={statusMapper.getClass(status)}>
              {statusMapper.getText(status)}
            </Badge>
          );
        },
      },
      {
        accessorKey: "user_data",
        header: "用户信息",
        size: 120,
        cell: ({ row }) => {
          return (
            <UserDataTooltip
              userData={row.original.user_data}
              className={cn("text-xs")}
            />
          );
        },
      },
      {
        accessorKey: "sub_app_count",
        header: "子应用总数",
        cell: ({ row }) => {
          const app = row.original;
          if (!app.sup_app || !app.sub_app_count) {
            return <span className="text-muted-foreground">未开通</span>;
          }
          const totalCount = app.sub_app_count.enable || 0;
          if (totalCount === 0) {
            return <span className="text-muted-foreground">未开通</span>;
          }
          return <div className="text-sm font-medium">{totalCount}</div>;
        },
      },
      {
        accessorKey: "change_time",
        header: "更新时间",
        cell: ({ getValue }) => {
          const date = getValue<Date>();
          const timeElement = formatTime(date, TIME_STYLE.RELATIVE_ELEMENT);
          return (
            <div className="text-xs text-muted-foreground">{timeElement}</div>
          );
        },
      },
      {
        id: "actions",
        header: () => <div className="text-center">详细</div>,
        size: 80,
        cell: ({ row }) => {
          const app = row.original;

          return (
            <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn("h-7", isMobile ? "px-1.5" : "px-2")}
                  onClick={() => handleViewDetail(app.id)}
                  title="查看详情"
                >
                  <Eye className={cn(isMobile ? "mr-1 h-3.5 w-3.5" : "mr-2 h-4 w-4")} />
                  {isMobile ? "详情" : "查看详情"}
                </Button>
              </DataTableActionItem>
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn("h-7", isMobile ? "px-1.5" : "px-2")}
                  onClick={() => handleViewSubApps(app.id)}
                  title="子应用列表"
                >
                  <List className={cn(isMobile ? "mr-1 h-3.5 w-3.5" : "mr-2 h-4 w-4")} />
                  {isMobile ? "子应用" : "子应用列表"}
                </Button>
              </DataTableActionItem>
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn("h-7", isMobile ? "px-1.5" : "px-2")}
                  onClick={() => handleViewRequests(app.id)}
                  title="请求列表"
                >
                  <List className={cn(isMobile ? "mr-1 h-3.5 w-3.5" : "mr-2 h-4 w-4")} />
                  {isMobile ? "请求" : "请求列表"}
                </Button>
              </DataTableActionItem>
            </DataTableAction>
          );
        },
      },
    ],
    [
      statusMapper,
      handleViewDetail,
      handleViewSubApps,
      isMobile,
      handleViewRequests,
    ],
  );

  return (
    <>
      <div className="container mx-auto p-4 max-w-[1600px] flex flex-col min-h-0 space-y-5">
        {/* 搜索和过滤 */}
        <div className="flex-shrink-0 mb-1 sm:mb-4">
          <FilterContainer
            defaultValues={{
              app_name: filterParam.app_name,
              status: filterParam.status?.toString(),
              user_id: filterParam.user_id?.toString(),
              client_id: filterParam.client_id,
              parent_app_id: filterParam.parent_app_id?.toString(),
              app_id: filterParam.app_id?.toString(),
            }}
            resolver={zodResolver(AdminAppListFilterFormSchema) as any}
            onSubmit={(data) => {
              // zod schema 已经处理了类型转换和空值清理，直接使用数据
              navigate({
                search: { ...data, page: 1 } as any,
              });
            }}
            onReset={() => {
              navigate({
                search: { page: 1, limit: currentLimit } as any,
              });
            }}
            countComponent={<FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />}
            className={cn("bg-card rounded-lg border shadow-sm relative")}
          >
            {(layoutParams, form) => (
              <div className="flex-1 flex flex-wrap items-end gap-3">
                {/* 应用名称过滤 */}
                <FilterInput
                  name="app_name"
                  placeholder="搜索应用名称"
                  label="应用名称"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                  className={cn(layoutParams.isMobile ? "w-full" : "w-40")}
                />

                {/* 状态过滤 */}
                {dictData.app_status && (
                  <FilterDictSelect
                    name="status"
                    placeholder="选择状态"
                    label="应用状态"
                    disabled={isLoading}
                    dictData={dictData.app_status}
                    layoutParams={layoutParams}
                    allLabel="全部"
                    className={cn(layoutParams.isMobile ? "w-full" : "w-32")}
                  />
                )}

                {/* 用户ID过滤 */}
                <FilterInput
                  name="user_id"
                  placeholder="输入用户ID"
                  type="number"
                  label="用户ID"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                  className={cn(layoutParams.isMobile ? "w-full" : "w-32")}
                />

                {/* Client ID 过滤 */}
                <FilterInput
                  name="client_id"
                  placeholder="输入Client ID"
                  label="Client ID"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                  className={cn(layoutParams.isMobile ? "w-full" : "w-40")}
                />

                {/* 应用ID过滤 */}
                <FilterInput
                  name="app_id"
                  placeholder="输入应用ID"
                  type="number"
                  label="应用ID"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                  className={cn(layoutParams.isMobile ? "w-full" : "w-32")}
                />

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

        {/* 表格和分页容器 */}
        <div className="flex-1 flex flex-col overflow-hidden min-h-0">
          {/* 数据表格 */}
          <div className={cn("flex-1 min-h-0")}>
            <div className="h-full">
              <DataTable
                data={apps}
                columns={columns}
                loading={isLoading}
                error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}
                className={cn("h-full [&_.data-table-row]:h-12 [&_td]:py-2 [&_th]:py-2 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b [&_.data-table-wrapper]:overflow-auto [&_.data-table-wrapper]:h-full", isMobile && "border-0 rounded-none")}
              />
            </div>
          </div>

          {/* 分页控件 - 始终显示 */}
          <div className="flex-shrink-0 pt-4">
            <PagePagination
              currentPage={filterParam.page || 1}
              pageSize={filterParam.limit || DEFAULT_PAGE_SIZE}
              total={countNumManager.getTotal() ?? 0}
              loading={isLoading}
              onChange={handlePageChange}
              onRefresh={refreshData}
              showRefresh={true}
            />
          </div>
        </div>

        {/* 应用详情抽屉 */}
        <AppDetailDrawer
          appId={detailDrawer.appId}
          open={detailDrawer.open}
          onClose={handleCloseDetailDrawer}
          appStatusDict={dictData.app_status}
        />

        {/* 子应用列表抽屉 */}
        <ListSubAppDrawer
          appId={subAppDrawer.appId}
          open={subAppDrawer.open}
          onClose={handleCloseSubAppDrawer}
          appStatusDict={dictData.app_status}
        />
      </div>
    </>
  );
}

// 导出 schema 供路由使用
export { AdminAppListFilterParamSchema } from "./list-schema";

