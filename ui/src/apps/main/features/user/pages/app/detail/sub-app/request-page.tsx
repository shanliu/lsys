import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { UserDataTooltip } from "@apps/main/components/local/user-data-tooltip";
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import { SubAppRequestDataDisplay } from "@apps/main/features/user/components/ui/sub-app-request-data-display";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import {
  DEFAULT_PAGE_SIZE,
  useCountNumManager,
} from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/sub-app/request";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  appSubRequestList,
  type AppSubRequestItemType,
} from "@shared/apis/user/app";
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
import { type ColumnDef } from "@tanstack/react-table";
import { Eye, ShieldCheck } from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { requestPromptModuleConfig } from "../nav-info";
import { SubAppRequestAuditActionDrawer } from "./request-audit-action-drawer";
import { SubAppRequestAuditInfoDrawer } from "./request-audit-info-drawer";
import {
  SubAppRequestFilterFormSchema
} from "./request-schema";

export function SubAppRequestPage() {
  // docs\api\user\app\sub_request_list.md
  // docs\api\user\app\confirm_exter_feature.md
  // docs\api\user\app\confirm.md
  // docs\api\user\app\oauth_server_client_confirm.md
  // docs\api\user\app\oauth_server_client_scope_confirm.md

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
        variant="page"
        error={dictErrors}
        onReset={refetchDict}
      />
    );
  }

  // 如果字典加载中，显示骨架屏
  if (dictIsLoading) {
    return <PageSkeletonTable variant="page" />;
  }

  // 字典加载成功，渲染内容组件
  return (
    <AppDetailNavContainer {...requestPromptModuleConfig}>
      <SubAppRequestContent dictData={dictData} />
    </AppDetailNavContainer>
  );
}

// 内容组件：负责内容加载和渲染
interface SubAppRequestContentProps {
  dictData: TypedDictData<["user_app"]>;
}

function SubAppRequestContent({ dictData }: SubAppRequestContentProps) {
  const { appId } = Route.useParams();
  const navigate = useNavigate();
  const isMobile = useIsMobile();
  const queryClient = useQueryClient();

  // 从 URL 搜索参数获取过滤条件
  const filterParam = Route.useSearch();

  const currentPage = filterParam.page || 1;
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 过滤条件
  const filters = {
    id: filterParam.id ?? null,
    sub_app_id: filterParam.sub_app_id ?? null,
    status: filterParam.status ?? null,
  };

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  // 获取子应用请求列表数据
  const { data: requestData, isSuccess, isLoading, isError, error } = useQuery({
    queryKey: [
      "appSubRequestList",
      appId,
      filterParam.id,
      filterParam.sub_app_id,
      filterParam.status,
      currentPage,
      currentLimit,
    ],
    queryFn: async ({ signal }) => {
      const result = await appSubRequestList(
        {
          app_id: Number(appId),
          id: filterParam.id,
          sub_app_id: filterParam.sub_app_id,
          status: filterParam.status,
          page: {
            page: currentPage,
            limit: currentLimit,
          },
          count_num: countNumManager.getCountNum(),
        },
        { signal }
      );
      return result;
    },
    placeholderData: (previousData) => previousData,
  });

  // 处理 Page 分页查询结果（自动提取 total）
  isSuccess && countNumManager.handlePageQueryResult(requestData);

  // 从查询结果中提取数据
  let requests = getQueryResponseData<AppSubRequestItemType[]>(requestData, []);

  // 客户端过滤请求类型（API不支持该参数）
  if (filterParam.request_type !== undefined && filterParam.request_type !== null) {
    requests = requests.filter((req) => req.request_type === filterParam.request_type);
  }

  // 刷新数据
  const refreshData = useCallback(() => {
    queryClient.refetchQueries({ queryKey: ["appSubRequestList"] });
  }, [queryClient]);

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["appSubRequestList"] });
  };

  // 对话框状态管理
  const [selectedRequestForInfo, setSelectedRequestForInfo] =
    useState<AppSubRequestItemType | null>(null);
  const [selectedRequestForAudit, setSelectedRequestForAudit] =
    useState<AppSubRequestItemType | null>(null);

  // 获取状态显示文本
  const getStatusText = useCallback(
    (status: number | string | undefined) => {
      if (status === undefined) return "未知";
      const statusKey = String(status);
      return dictData.request_status?.getLabel(statusKey) || statusKey;
    },
    [dictData.request_status],
  );

  // 获取请求类型显示文本
  const getRequestTypeText = useCallback(
    (type: number | string) => {
      const typeKey = String(type);
      return dictData.request_type?.getLabel(typeKey) || typeKey;
    },
    [dictData.request_type],
  );

  // 状态样式映射
  const statusMapper = useMemo(
    () =>
      createStatusMapper(
        { 1: "warning", 2: "success", 3: "danger", 4: "neutral" },
        getStatusText,
      ),
    [getStatusText],
  );

  // 检查请求类型是否支持审核（只有 1, 2, 6, 7, 8 支持）
  const canAudit = useCallback((requestType: number) => {
    return [1, 2, 6, 7, 8].includes(requestType);
  }, []);

  // 渲染附加数据
  const renderAdditionalData = useCallback(
    (request: AppSubRequestItemType) => {
      return (
        <div className="px-4 py-3 space-y-2">
          <SubAppRequestDataDisplay data={request} showLabel={true} mode="table" />
        </div>
      );
    },
    [],
  );
  // 定义表格列
  const columns = useMemo<ColumnDef<AppSubRequestItemType>[]>(() => {
    const baseColumns: ColumnDef<AppSubRequestItemType>[] = [
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
        header: "应用名(应用ID)",
        size: 180,
        cell: ({ getValue, row }) => {
          const appName = getValue() as string | null;
          const appId = row.original.app_id;
          return (
            <div className="text-xs">
              {appName ? (
                <>
                  {appName} (<CopyableText value={String(appId)} showIcon={false} />)
                </>
              ) : (
                <>(<CopyableText value={String(appId)} showIcon={false} />)</>
              )}
            </div>
          );
        },
      },
      {
        accessorKey: "app_client",
        header: "应用标识",
        size: 150,
        cell: ({ getValue }) => (
          <div className="font-mono text-xs">{getValue() as string || '-'}</div>
        ),
      },
      {
        accessorKey: "request_type",
        header: "请求类型",
        size: 120,
        cell: ({ getValue }) => (
          <Badge variant="outline" className={cn("text-xs")}>
            {getRequestTypeText(getValue() as number)}
          </Badge>
        ),
      },
      {
        accessorKey: "status",
        header: "状态",
        size: 80,
        cell: ({ getValue }) => {
          const status = getValue() as number | string | undefined;
          const statusNum = Number(status) || 0;
          return (
            <Badge variant="secondary" className={cn(statusMapper.getClass(statusNum))}>
              {statusMapper.getText(statusNum)}
            </Badge>
          );
        },
      },
      {
        accessorKey: "user_data",
        header: "请求用户",
        size: 80,
        cell: ({ getValue }) => (
          <UserDataTooltip userData={getValue() as any} className="text-xs" />
        ),
      },
      {
        accessorKey: "request_time",
        header: "请求时间",
        cell: ({ getValue, row }) => {
          return (
            <div className="text-xs text-muted-foreground">
              {formatTime(getValue() as string, TIME_STYLE.RELATIVE_ELEMENT)}
            </div>
          );
        },
      },
      {
        id: "actions",
        header: () => <div className="text-center">操作</div>,
        cell: ({ row }) => {
          const request = row.original;
          const isPending = request.status === 1;
          const isAuditable = canAudit(request.request_type);

          // 未审核状态且支持审核类型：显示审核按钮
          if (isPending && isAuditable) {
            return (
              <DataTableAction
                className={cn(isMobile ? "justify-end" : "justify-center")}
              >
                <DataTableActionItem
                  mobileDisplay="display"
                  desktopDisplay="display"
                >
                  <Button
                    size="sm"
                    onClick={() => setSelectedRequestForAudit(request)}
                    variant="ghost"
                    className={cn("h-7")}
                    title="审核申请"
                  >

                    <>
                      <ShieldCheck className="h-4 w-4" />
                      {isMobile && <span className="ml-2">审核请求</span>}
                    </>

                  </Button>
                </DataTableActionItem>
              </DataTableAction>
            );
          }

          // 其他状态：显示详情按钮
          return (
            <DataTableAction
              className={cn(isMobile ? "justify-end" : "justify-center")}
            >
              <DataTableActionItem
                mobileDisplay="display"
                desktopDisplay="display"
              >
                <Button
                  onClick={() => setSelectedRequestForInfo(request)}
                  variant="ghost"
                  size="sm"
                  className={cn("h-7")}
                  title="查看详情"
                >
                  <Eye className="h-4 w-4" />
                  {isMobile && <span className="ml-2">查看详情</span>}
                </Button>
              </DataTableActionItem>
            </DataTableAction>
          );
        },
      },
    ];

    return baseColumns;
  }, [statusMapper, getRequestTypeText, isMobile, canAudit]);

  return (
    <div className="flex flex-col min-h-0 space-y-3">
      <div className="flex-shrink-0 mb-1 sm:mb-4">
        {/* 过滤器 */}
        <FilterContainer
          defaultValues={{
            id: filterParam.id?.toString(),
            sub_app_id: filterParam.sub_app_id?.toString(),
            request_type: filterParam.request_type?.toString(),
            status: filterParam.status?.toString(),
          }}
          resolver={zodResolver(SubAppRequestFilterFormSchema) as any}
          onSubmit={(data) => {
            navigate({
              search: { ...data, page: 1, limit: currentLimit } as any,
            });
          }}
          onReset={() => {
            navigate({
              search: { page: 1, limit: currentLimit } as any,
            });
          }}
          countComponent={
            <FilterTotalCount
              total={countNumManager.getTotal() ?? 0}
              loading={isLoading}
            />
          }
          className="bg-card rounded-lg border shadow-sm relative"
        >
          {(layoutParams, form) => (
            <div className="flex-1 flex flex-wrap items-end gap-3">
              {/* 请求ID过滤 */}
              <FilterInput
                name="id"
                placeholder="输入请求ID"
                type="number"
                label="请求ID"
                disabled={isLoading}
                layoutParams={layoutParams}
              />

              {/* 子应用ID过滤 */}
              <FilterInput
                name="sub_app_id"
                placeholder="输入子应用ID"
                type="number"
                label="子应用ID"
                disabled={isLoading}
                layoutParams={layoutParams}
              />

              {/* 请求类型过滤 */}
              {dictData.request_type && (
                <FilterDictSelect
                  name="request_type"
                  placeholder="选择类型"
                  label="请求类型"
                  disabled={isLoading}
                  dictData={dictData.request_type}
                  layoutParams={layoutParams}
                  allLabel="全部"
                />
              )}

              {/* 状态过滤 */}
              {dictData.request_status && (
                <FilterDictSelect
                  name="status"
                  placeholder="选择状态"
                  label="请求状态"
                  disabled={isLoading}
                  dictData={dictData.request_status}
                  layoutParams={layoutParams}
                  allLabel="全部"
                />
              )}

              {/* 动作按钮区域 */}
              <FilterActions
                form={form}
                loading={isLoading}
                layoutParams={layoutParams}
                onRefreshSearch={clearCacheAndReload}
              />
            </div>
          )}
        </FilterContainer>
      </div>

      {/* 表格和分页容器 - 确保不超出页面高度 */}
      <div className="flex-1 flex flex-col min-h-0">
        {/* 数据表格 - 使用 flex-1 但不设置 min-h-0，让分页有足够空间 */}
        <div className="flex-1 overflow-hidden">
          <DataTable
            data={requests}
            columns={columns}
            loading={isLoading}
            error={
              isError ? (
                <CenteredError
                  error={error}
                  variant="content"
                  onReset={refreshData}
                />
              ) : null
            }
            expandedRowRender={renderAdditionalData}
            scrollSnapDelay={300}
            className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
            tableContainerClassName="h-full"
          />
        </div>

        {/* 分页控件 - 始终显示 */}
        <div className="flex-shrink-0 pt-4 pb-4">
          <PagePagination
            currentPage={currentPage}
            pageSize={currentLimit}
            total={countNumManager.getTotal() ?? 0}
            loading={isLoading}
            onChange={(page: number) => {
              navigate({
                search: { ...filterParam, page } as any,
              });
            }}
            onPageSizeChange={(limit: number) => {
              navigate({
                search: { ...filterParam, page: 1, limit } as any,
              });
            }}
          />
        </div>
      </div>

      {/* 审核信息对话框 */}
      <SubAppRequestAuditInfoDrawer
        request={selectedRequestForInfo}
        open={!!selectedRequestForInfo}
        onOpenChange={(open) => !open && setSelectedRequestForInfo(null)}
        requestTypeDict={dictData.request_type}
        requestStatusDict={dictData.request_status}
      />

      {/* 审核操作对话框 */}
      <SubAppRequestAuditActionDrawer
        request={selectedRequestForAudit}
        open={!!selectedRequestForAudit}
        onOpenChange={(open) => !open && setSelectedRequestForAudit(null)}
        onAuditComplete={refreshData}
        requestTypeDict={dictData.request_type}
        requestStatusDict={dictData.request_status}
      />
    </div>
  );
}

// 导出 schema 供路由使用
export { SubAppRequestFilterParamSchema } from './request-schema';
