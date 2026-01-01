import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { PermGuard } from "@apps/main/components/local/perm-guard";
import { UserDataTooltip } from "@apps/main/components/local/user-data-tooltip";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import { DEFAULT_PAGE_SIZE, useCountNumManager } from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/admin/app/request";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  appRequestList,
  type AppRequestItemType,
} from "@shared/apis/admin/app";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { PagePagination } from "@shared/components/custom/pagination";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
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
import { Link, useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import { Eye, ShieldCheck } from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { AppRequestDataDisplay } from "../../components/ui/app-request-data-display";
import { hasRequestData } from "../../components/ui/app-request-utils";
import { RequestAuditActionDrawer } from "./request-audit-action-drawer";
import { RequestAuditInfoDrawer } from "./request-audit-info-drawer";
import {
  AdminAppRequestFilterFormSchema
} from "./request-schema";

export function AppRequestPage() {
  // system\app\mapping.md
  // system\app\request_list.md
  // system\app\confirm_exter_feature.md
  // system\app\confirm_inner_feature_exter_login_confirm.md
  // system\app\confirm_inner_feature_sub_app_confirm.md
  // system\app\confirm.md
  // system\app\oauth_client_confirm.md
  // system\app\oauth_client_scope_confirm.md
  // system\app\oauth_server_confirm.md
  // 字典数据获取 - 统一在最顶层获取一次
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
    return <PageSkeletonTable variant="page" className={cn("m-4 md:m-6")} />;
  }

  // 字典加载成功，渲染内容组件
  return <AppRequestContent dictData={dictData} />;
}

// 内容组件：负责内容加载和渲染
interface AppRequestContentProps {
  dictData: TypedDictData<["admin_app"]>;
}

function AppRequestContent({ dictData }: AppRequestContentProps) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();

  // 获取 URL search 参数
  const filterParam = Route.useSearch();

  const currentPage = filterParam.page || 1;
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 过滤条件
  const filters = {
    id: filterParam.id ?? null,
    app_id: filterParam.app_id ?? null,
    request_type: filterParam.request_type || null,
    status: filterParam.status || null,
  };

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  // 获取请求列表数据
  const { data: requestData, isSuccess, isLoading, isError, error } = useQuery({
    queryKey: [
      "appRequestList",
      currentPage,
      currentLimit,
      filterParam.id,
      filterParam.app_id,
      filterParam.request_type,
      filterParam.status,
    ],
    queryFn: ({ signal }) =>
      appRequestList(
        {
          page: {
            page: currentPage,
            limit: currentLimit,
          },
          count_num: countNumManager.getCountNum(),
          ...(filterParam.id !== undefined && { id: filterParam.id }),
          ...(filterParam.app_id !== undefined && { app_id: filterParam.app_id }),
          ...(filterParam.request_type && { request_type: filterParam.request_type }),
          ...(filterParam.status && { status: filterParam.status }),
        },
        { signal },
      ),
    placeholderData: (previousData) => previousData,
  });

  // 处理 Page 分页查询结果（自动提取 total）
  isSuccess && countNumManager.handlePageQueryResult(requestData);

  // 刷新数据
  const refreshData = useCallback(() => {
    // 使用 refetchQueries 强制重新加载，即使查询失败也能触发
    queryClient.refetchQueries({ queryKey: ["appRequestList"] });
  }, [queryClient]);

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["appRequestList"] });
  };

  // 处理分页变化
  const handlePageChange = (page: number) => {
    navigate({
      search: { ...filterParam, page } as any,
    });
  };

  const requests = getQueryResponseData<AppRequestItemType[]>(requestData, []);

  // 对话框状态管理
  const [selectedRequestForInfo, setSelectedRequestForInfo] =
    useState<AppRequestItemType | null>(null);
  const [selectedRequestForAudit, setSelectedRequestForAudit] =
    useState<AppRequestItemType | null>(null);

  // 获取状态显示文本
  const getStatusText = useCallback(
    (status: number | string | undefined) => {
      if (status === undefined) return "未知";
      const statusKey = String(status);
      return dictData.request_status.getLabel(statusKey) || statusKey;
    },
    [dictData.request_status],
  );

  // 获取请求类型显示文本
  const getRequestTypeText = useCallback(
    (type: number | string) => {
      const typeKey = String(type);
      return dictData.request_type.getLabel(typeKey) || typeKey;
    },
    [dictData.request_type],
  );

  // 状态样式映射
  const statusMapper = useMemo(
    () => createStatusMapper(
      { 1: "warning", 2: "success", 3: "danger", 4: "neutral" },
      getStatusText,
    ),
    [getStatusText]
  );

  // 渲染附加数据
  const renderAdditionalData = useCallback((request: AppRequestItemType) => {
    if (!hasRequestData(request)) {
      return null;
    }

    return (
      <div className="px-4 py-3 space-y-2">
        <AppRequestDataDisplay data={request} showLabel={true} mode="table" />
      </div>
    );
  }, []);

  // 定义表格列
  const columns = useMemo<ColumnDef<AppRequestItemType>[]>(() => {
    const baseColumns: ColumnDef<AppRequestItemType>[] = [
      {
        accessorKey: "id",
        header: () => <div className={cn(isMobile ? "" : "text-right")}>ID</div>,
        size: 80,
        cell: ({ getValue }) => (
          <div className={cn("font-mono text-xs", isMobile ? "" : "text-right")}>{getValue() as number}</div>
        ),
      },
      {
        accessorKey: "app_name",
        header: "应用名",
        cell: ({ row }) => {
          const request = row.original;
          if (!request.app_name || !request.app_id) {
            return <span className="text-muted-foreground">-</span>;
          }
          return (
            <div className="flex items-center gap-2">
              <Link
                to="/admin/app/list"
                search={{ app_id: request.app_id, detail: 1 } as any}
                className="text-primary hover:underline font-medium"
              >
                {request.app_name}
              </Link>
              <span className="text-xs text-muted-foreground">
                AppId:{request.app_id}
              </span>
            </div>
          );
        },
      },
      {
        accessorKey: "app_client",
        header: "应用标识",
        size: 120,
        cell: ({ getValue }) => (
          getValue() as string
        ),
      },
      {
        accessorKey: "request_type",
        header: "请求类型",
        size: 140,
        cell: ({ getValue }) => (
          <Badge variant="outline" className="text-xs">
            {getRequestTypeText(getValue() as number)}
          </Badge>
        ),
      },
      {
        accessorKey: "status",
        header: "状态",
        size: 100,
        cell: ({ getValue }) => {
          const status = getValue() as number | string | undefined;
          const statusNum = Number(status) || 0;
          return (
            <Badge variant="secondary" className={statusMapper.getClass(statusNum)}>
              {statusMapper.getText(statusNum)}
            </Badge>
          );
        },
      },
      {
        accessorKey: "request_user_data",
        header: "请求用户",
        size: 120,
        cell: ({ getValue }) => (
          <UserDataTooltip
            userData={getValue() as any}
            className="text-xs"
          />
        ),
      },
      {
        accessorKey: "request_time",
        header: "请求时间",
        cell: ({ getValue, row }) => {
          const request = row.original;
          return (
            <div className="text-xs text-muted-foreground">
              <div>
                {formatTime(getValue() as string, TIME_STYLE.RELATIVE_ELEMENT)}
              </div>
              {/* 移动端显示用户信息 */}
              <div className="md:hidden mt-1 text-muted-foreground/70">
                <UserDataTooltip
                  userData={request.request_user_data}
                  className="text-xs"
                />
              </div>
            </div>
          );
        },
      },
      {
        accessorKey: "confirm_time",
        header: "审核时间",
        size: 140,
        cell: ({ getValue }) => (
          <div className="text-xs text-muted-foreground">
            {getValue() ? (
              formatTime(getValue() as string, TIME_STYLE.RELATIVE_ELEMENT)
            ) : (
              <span className="text-muted-foreground/50">-</span>
            )}
          </div>
        ),
      },
      {
        id: "actions",
        header: () => <div className="text-center">操作</div>,
        cell: ({ row }) => {
          const request = row.original;
          const isPending = request.status === 1;
          // 未审核状态：显示审核按钮
          if (isPending) {
            return (
              <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
                <DataTableActionItem mobileDisplay="display" desktopDisplay="display">
                  <PermGuard
                    permission="admin:app:request:confirm"
                    fallback="disabled"
                  >
                    <Button
                      size="sm"
                      onClick={() => setSelectedRequestForAudit(request)}
                      variant="ghost"
                      className={cn("h-7")}
                      title="审核申请"
                    >
                      <ShieldCheck className="h-4 w-4" />
                      {isMobile ? <span className="ml-2">审核应用</span> : ""}
                    </Button>
                  </PermGuard>
                </DataTableActionItem>
              </DataTableAction>
            );
          }

          // 其他状态：显示详情按钮
          return (
            <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
              <DataTableActionItem mobileDisplay="display" desktopDisplay="display">
                <Button
                  onClick={() => setSelectedRequestForInfo(request)}
                  variant="ghost"
                  size="sm"
                  className={cn("h-7")}
                  title="查看详情"
                >
                  <Eye className=" h-4 w-4" />
                  {isMobile ? <span className="ml-2">查看详情</span> : ""}
                </Button>
              </DataTableActionItem>
            </DataTableAction>
          );
        },
      },
    ];

    return baseColumns;
  }, [
    statusMapper,
    getRequestTypeText,
    isMobile
  ]);

  return (
    <div className="container mx-auto p-4 max-w-[1600px] flex flex-col min-h-0 space-y-5">
      {/* 搜索和过滤 */}
      <div className="flex-shrink-0 mb-1 sm:mb-4">
        <FilterContainer
          defaultValues={{
            id: filterParam.id?.toString(),
            app_id: filterParam.app_id?.toString(),
            request_type: filterParam.request_type,
            status: filterParam.status,
          }}
          resolver={zodResolver(AdminAppRequestFilterFormSchema) as any}
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
          countComponent={
            <FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />
          }
          className={cn("bg-card rounded-lg border shadow-sm relative")}
        >
          {(layoutParams, form) => (
            <div className="flex-1 flex flex-wrap items-end gap-3">
              {/* ID过滤 */}
              <FilterInput
                name="id"
                placeholder="输入请求ID"
                type="number"
                label="请求ID"
                disabled={isLoading}
                layoutParams={layoutParams}
              />

              {/* 应用ID过滤 */}
              <FilterInput
                name="app_id"
                placeholder="输入应用ID"
                type="number"
                label="应用ID"
                disabled={isLoading}
                layoutParams={layoutParams}
              />

              {/* 请求类型过滤 */}
              <FilterDictSelect
                name="request_type"
                placeholder="选择类型"
                label="请求类型"
                disabled={isLoading}
                dictData={dictData.request_type}
                layoutParams={layoutParams}
                allLabel="全部"
              />

              {/* 状态过滤 */}
              <FilterDictSelect
                name="status"
                placeholder="选择状态"
                label="请求状态"
                disabled={isLoading}
                dictData={dictData.request_status}
                layoutParams={layoutParams}
                allLabel="全部"
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
        <div className="flex-1 min-h-0">
          <div className="h-full">
            <DataTable
              data={requests}
              columns={columns}
              loading={isLoading}
              error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}
              expandedRowRender={renderAdditionalData}
              rightStickyColumns={[{ column: "actions", minWidth: "80px", maxWidth: "100px" }]}
              className={cn("h-full [&_.data-table-row]:h-12 [&_td]:py-2 [&_th]:py-2 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b [&_.data-table-wrapper]:overflow-auto [&_.data-table-wrapper]:h-full")}
            />

            <RequestAuditInfoDrawer
              request={selectedRequestForInfo}
              open={!!selectedRequestForInfo}
              onOpenChange={(open) => !open && setSelectedRequestForInfo(null)}
              requestTypeDict={dictData.request_type}
              requestStatusDict={dictData.request_status}
            />

            <RequestAuditActionDrawer
              request={selectedRequestForAudit}
              open={!!selectedRequestForAudit}
              onOpenChange={(open) => !open && setSelectedRequestForAudit(null)}
              onAuditComplete={refreshData}
              requestTypeDict={dictData.request_type}
              requestStatusDict={dictData.request_status}
            />
          </div>
        </div>

        {/* 分页控件 - 始终显示 */}
        <div className="flex-shrink-0 pt-4">
          <PagePagination
            currentPage={currentPage}
            pageSize={currentLimit}
            total={countNumManager.getTotal() ?? 0}
            loading={isLoading}
            onChange={handlePageChange}
            onRefresh={refreshData}
            showRefresh={true}
            pageSizeOptions={[20, 50, 100]}
            showPageSize={true}
          />
        </div>
      </div>
    </div>
  );
}

// 导出 schema 供路由使用
export { AdminAppRequestFilterParamSchema } from "./request-schema";

