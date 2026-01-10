import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterSystemAppSelector } from "@apps/main/components/filter-container/filter-system-app-selector";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { AuditDetailTooltip } from "@apps/main/components/local/audit-detail-tooltip";
import { UserDataTooltip } from "@apps/main/components/local/user-data-tooltip";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import {
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  useCountNumManager,
  useOffsetPaginationHandlers,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/admin/rbac/audit-log";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  rbacAuditData,
  type AuditDataType,
} from "@shared/apis/admin/rbac-base";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { OffsetPagination } from "@shared/components/custom/pagination";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useIsMobile } from "@shared/hooks/use-mobile";
import {
  cn,
  extractMinMax,
  formatTime,
  getQueryResponseData,
  getQueryResponseNext,
  TIME_STYLE,
} from "@shared/lib/utils";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ColumnDef } from "@tanstack/react-table";
import {
  Eye
} from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { AuditLogDetailDrawer } from "./audit-log-detail-drawer";
import {
  RbacAuditLogFilterFormSchema,
  type RbacOffsetPaginationType
} from "./audit-log-schema";

export function AuditLogPage() {
  // 字典数据获取 - 统一在最顶层获取一次
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["admin_rbac"] as const);

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
    return (
      <PageSkeletonTable
        variant="page"
        rows={6}
        columns={8}
        className={cn("container mx-auto m-4 md:m-6")}
      />
    );
  }

  // 字典加载成功，渲染内容组件
  return <AuditLogContent dictData={dictData} />;
}

// 内容组件：负责内容加载和渲染
interface AuditLogContentProps {
  dictData: TypedDictData<["admin_rbac"]>;
}

function AuditLogContent({ dictData }: AuditLogContentProps) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();

  // 获取 URL search 参数
  const filterParam = Route.useSearch();

  // 详情对话框状态
  const [detailDialog, setDetailDialog] = useState({
    open: false,
    audit: null as AuditDataType | null,
  });

  // 过滤条件从 URL 参数获取
  const filters = {
    user_id: filterParam.user_id || null,
    app_id: filterParam.app_id || null,
    user_ip: filterParam.user_ip || null,
    request_id: filterParam.request_id || null,
  };

  // 分页状态 - 直接从 URL 参数派生，无需 useState
  const pagination: RbacOffsetPaginationType = {
    pos: filterParam.pos || null,
    limit: filterParam.limit || DEFAULT_PAGE_SIZE,
    forward: filterParam.forward || false,
    more: true,
    eq_pos: filterParam.eq_pos || false,
  };

  const currentLimit = pagination.limit;

  // 搜索导航函数
  const searchGo = useSearchNavigate(navigate, filterParam);

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  // 获取审计数据
  const { data: auditData, isSuccess: auditIsSuccess, isLoading: auditIsLoading, isError: auditIsError, error: auditError } = useQuery({
    queryKey: [
      "rbacAuditData",
      pagination.pos,
      pagination.limit,
      pagination.forward,
      pagination.eq_pos,
      filters.user_id,
      filters.app_id,
      filters.user_ip,
      filters.request_id,
    ],
    queryFn: ({ signal }) =>
      rbacAuditData(
        {
          limit: {
            eq_pos: pagination.eq_pos,
            pos: pagination.pos,
            limit: pagination.limit,
            forward: pagination.forward,
            more: pagination.more,
          },
          count_num: countNumManager.getCountNum(),
          ...(filters.user_id !== undefined &&
            filters.user_id !== null && { user_id: filters.user_id }),
          ...(filters.app_id !== undefined &&
            filters.app_id !== null && { app_id: filters.app_id }),
          ...(filters.user_ip && { user_ip: filters.user_ip }),
          ...(filters.request_id !== undefined &&
            filters.request_id !== null && { request_id: filters.request_id }),
        },
        { signal },
      ),
    placeholderData: (previousData) => previousData,
  });

  // 处理 Limit 分页查询结果（自动提取 total 和 next）
  auditIsSuccess && countNumManager.handleLimitQueryResult(auditData);

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["rbacAuditData"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["rbacAuditData"] });
  };

  // 获取API响应数据
  const audits = getQueryResponseData<AuditDataType[]>(auditData, []);
  const nextPageStartPos = getQueryResponseNext(auditData);

  const auditResultDict = dictData.audit_result;

  // 审计结果状态映射器
  // 字典: "1" -> 授权通过, "0" -> 授权失败
  const auditResultMapper = useMemo(
    () =>
      createStatusMapper<string>(
        { "1": "success", "0": "danger" },
        (status: string) => {
          return auditResultDict?.getLabel(status) || status;
        },
      ),
    [auditResultDict],
  );

  // 处理查看详情
  const handleViewDetail = useCallback((audit: AuditDataType) => {
    setDetailDialog({ open: true, audit });
  }, []);

  // 定义表格列
  const columns = useMemo<ColumnDef<AuditDataType>[]>(
    () => [
      {
        accessorKey: "audit.id",
        header: () => <div className={cn(isMobile ? "" : "text-right")}>ID</div>,
        size: 80,
        cell: ({ getValue }) => (
          <div className={cn("font-mono text-xs", isMobile ? "" : "text-right")}>{getValue<number>()}</div>
        ),
      },
      {
        accessorKey: "audit.check_result",
        header: "审计结果",
        size: 120,
        cell: ({ getValue }) => {
          const result = getValue<string>() ?? "";
          return (
            <Badge
              variant="secondary"
              className={auditResultMapper.getClass(result)}
            >
              {auditResultMapper.getText(result)}
            </Badge>
          );
        },
      },
      {
        accessorKey: "user",
        header: "用户",
        size: 120,
        cell: ({ getValue }) => {
          return <UserDataTooltip userData={getValue<any>()} />
        },
      },
      {
        accessorKey: "audit.user_ip",
        header: "用户IP",
        size: 130,
        cell: ({ getValue }) => {
          const ip = getValue<string>();
          return ip ? (
            <CopyableText value={ip} className={cn("text-xs whitespace-nowrap")} />
          ) : (
            <div className="font-mono text-xs">-</div>
          );
        },
      },
      {
        accessorKey: "audit.request_id",
        header: "请求ID",
        size: 200,
        cell: ({ getValue }) => {
          const requestId = getValue<string>();
          return (
            <div className="font-mono text-xs truncate" title={requestId}>
              {requestId || "-"}
            </div>
          );
        },
      },
      {
        accessorKey: "detail",
        header: "权限检测资源",
        size: 100,
        cell: ({ getValue }) => {
          return <AuditDetailTooltip details={getValue<any[]>()} />;
        },
      },
      {
        accessorKey: "audit.add_time",
        header: "授权时间",
        size: 180,
        cell: ({ getValue }) => {
          const date = getValue<Date>();
          const timeElement = formatTime(
            date,
            TIME_STYLE.ABSOLUTE_TEXT,
          );
          return (
            <div className="text-xs text-muted-foreground">{timeElement}</div>
          );
        },
      },
      {
        id: "actions",
        size: 80,
        header: () => <div className="text-center">详细</div>,
        cell: ({ row }) => {
          const audit = row.original;

          return (
            <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
              <DataTableActionItem mobileDisplay="display" desktopDisplay="display">
                <Button
                  onClick={() => handleViewDetail(audit)}
                  variant="ghost"
                  size="sm"
                  className={cn("h-7 px-2")}
                  title="查看详情"
                >
                  <Eye className="h-4 w-4" />
                  {isMobile ? <span className="mr-2">查看详情</span> : ""}

                </Button>
              </DataTableActionItem>
            </DataTableAction>
          );
        },
      },
    ],
    [auditResultMapper, handleViewDetail, isMobile],
  );

  // 使用统一的分页处理器
  const { handleNextPage, handlePrevPage, canGoNext, canGoPrev } =
    useOffsetPaginationHandlers({
      ...extractMinMax(audits.map(a => ({ id: a.audit.id })), 'id', 'minId', 'maxId'),
      pagination,
      nextPageStartPos,
      searchGo,
      defaultForward: false, // 从大到小排序（新到旧）
    });

  // 关闭详情对话框
  const handleCloseDetailDialog = () => {
    setDetailDialog({ open: false, audit: null });
  };

  const isLoading = auditIsLoading;

  return (
    <>
      <div className="container mx-auto p-4 max-w-[1600px] flex flex-col min-h-0 space-y-5">
        {/* 搜索和过滤 */}
        <div className="flex-shrink-0 mb-1 sm:mb-4">
          <FilterContainer
            defaultValues={{
              user_id: filterParam.user_id?.toString(),
              app_id: filterParam.app_id?.toString(),
              user_ip: filterParam.user_ip,
              request_id: filterParam.request_id?.toString(),
            }}
            resolver={zodResolver(RbacAuditLogFilterFormSchema) as any}
            onSubmit={(data) => {
              // zod schema 已经处理了类型转换和空值清理，直接使用数据
              navigate({
                search: { ...data, pos: null, forward: false, eq_pos: false } as any,
              });
            }}
            onReset={() => {
              navigate({
                search: {
                  pos: null,
                  forward: false,
                  eq_pos: false,
                  limit: currentLimit,
                } as any,
              });
            }}
            countComponent={<FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />}
            className="bg-card rounded-lg border shadow-sm relative"
          >
            {(layoutParams, form) => (
              <div className="flex-1 flex flex-wrap items-end gap-3 lg:gap-4">
                {/* 用户ID过滤 */}
                <div className="flex-1 min-w-[120px] max-w-[180px]">
                  <FilterInput
                    name="user_id"
                    placeholder="输入用户ID"
                    type="number"
                    label="用户ID"
                    disabled={isLoading}
                    layoutParams={layoutParams}
                  />
                </div>

                {/* 应用ID过滤 */}
                <div className="flex-1 min-w-[140px] max-w-[200px]">
                  <FilterSystemAppSelector
                    name="app_id"
                    placeholder="选择应用"
                    label="应用"
                    disabled={isLoading}
                    layoutParams={layoutParams}
                  />
                </div>

                {/* 用户IP过滤 */}
                <div className="flex-1 min-w-[140px] max-w-[200px]">
                  <FilterInput
                    name="user_ip"
                    placeholder="输入IP地址"
                    label="用户IP"
                    disabled={isLoading}
                    layoutParams={layoutParams}
                  />
                </div>

                {/* 请求ID过滤 */}
                <div className="flex-1 min-w-[120px] max-w-[180px]">
                  <FilterInput
                    name="request_id"
                    placeholder="输入请求ID"
                    type="number"
                    label="请求ID"
                    disabled={isLoading}
                    layoutParams={layoutParams}
                  />
                </div>

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
            <DataTable
              data={audits}
              columns={columns}
              loading={isLoading}
              error={auditIsError ? <CenteredError error={auditError} variant="content" onReset={refreshData} /> : null}
              className="h-full [&_.data-table-row]:h-12 [&_td]:py-2 [&_th]:py-2 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b [&_.data-table-wrapper]:overflow-auto [&_.data-table-wrapper]:h-full"
            />
          </div>

          {/* 分页控件 - 确保分页条始终显示 */}
          <div className="flex-shrink-0 pt-4">
            {(countNumManager.getTotal() ?? 0) > 0 && (
              <OffsetPagination
                limit={pagination.limit}
                hasNext={canGoNext}
                canGoPrev={canGoPrev}
                total={countNumManager.getTotal()}
                currentPageSize={audits.length}
                loading={isLoading}
                onPrevious={handlePrevPage}
                onNext={handleNextPage}
                onRefresh={refreshData}
                showRefresh={true}
                showPageSize={true}
                pageSizeOptions={PAGE_SIZE_OPTIONS}
                onPageSizeChange={(pageSize) => {
                  searchGo({
                    limit: pageSize,
                    pos: null,
                    forward: false,
                    eq_pos: false,
                  });
                }}
              />
            )}
          </div>
        </div>

        {/* 详情对话框 */}
        <AuditLogDetailDrawer
          audit={detailDialog.audit}
          open={detailDialog.open}
          onOpenChange={handleCloseDetailDialog}
          auditResultDict={auditResultDict}
        />
      </div>
    </>
  );
}
