import { FilterBar } from "@apps/main/components/filter-bar/container";
import { FilterActions } from "@apps/main/components/filter-bar/filter-actions/filter-actions";
import { FilterResetButton } from "@apps/main/components/filter-bar/filter-actions/filter-reset-button";
import { FilterSearchButton } from "@apps/main/components/filter-bar/filter-actions/filter-search-button";
import { FilterDictSelect, FilterInput, FilterTotalCount } from "@apps/main/components/filter-bar/filter-fields";
import { ExportButton, ExportMobileButton, ExportSplitButton } from "@apps/main/components/export-manager/export-buttons";
import { ExportDrawer } from "@apps/main/components/export-manager/export-drawer";
import { useAdminExportAction } from "@apps/main/hooks/use-admin-export-action";
import { useFilterBarForm } from "@apps/main/hooks/use-filter-bar-form";
import * as z from "zod";
import {
  useDictData,
  type TypedDictData,
} from "@apps/main/hooks/use-dict-data";
import {
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  useLimitCountNum,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { Route } from "@apps/main/routes/_main/admin/user/change-log";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  systemUserChangeLogs,
  type SystemUserChangeLogItemType,
  type SystemUserChangeLogsParamType,
} from "@shared/apis/admin/user";
import { EXPORT_TYPE_SYSTEM_USER_CHANGE_LOG } from "@shared/apis/admin/export";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { CursorPagination } from "@shared/components/custom/pagination";
import {
  DataTable,
  DataTableAction,
  DataTableActionItem,
} from "@shared/components/custom/table";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useIsMobile } from "@shared/hooks/use-mobile";
import {
  cn,
  formatTime,
  getQueryResponseCursor,
  getQueryResponseData,
  TIME_STYLE,
} from "@shared/lib/utils";
import { formatTotalCount } from "@shared/lib/utils/format-utils";
import { type LimitType } from "@shared/types/base-schema";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ColumnDef } from "@tanstack/react-table";
import { Eye } from "lucide-react";
import { useMemo, useState } from "react";
import { ChangeLogDetailDrawer } from "./change-log-detail-drawer";
import { UserChangeLogFilterFormSchema } from "./change-log-schema";
import { UserDataTooltip } from "@apps/main/components/local/user-data-tooltip";

export function ChangeLogPage() {
  // system\user\mapping.md
  //system\user\change_logs.md

  // 字典数据获取 - 统一在最顶层获取一次
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["admin_user", "admin_export"] as const);

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
        columns={6}
        className={cn("container mx-auto m-4 md:m-6")}
      />
    );
  }

  // 字典加载成功，渲染内容组件
  return <ChangeLogContent dictData={dictData} />;
}

// 内容组件：负责内容加载和渲染
interface ChangeLogContentProps {
  dictData: TypedDictData<["admin_user", "admin_export"]>;
}

function ChangeLogContent({ dictData }: ChangeLogContentProps) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();

  // 获取 URL search 参数
  const filterParam = Route.useSearch();

  // 详情对话框状态
  const [detailDialog, setDetailDialog] = useState({
    open: false,
    log: null as SystemUserChangeLogItemType | null,
  });

  // 过滤条件从 URL 参数获取
  const filters = {
    log_type: filterParam.log_type || null,
    add_user_id: filterParam.add_user_id || null,
  };

  // 导出操作 hook
  const exportAction = useAdminExportAction({
    exportType: EXPORT_TYPE_SYSTEM_USER_CHANGE_LOG,
    params: {
      log_type: filters.log_type ?? undefined,
      add_user_id: filters.add_user_id ?? undefined,
    },
  });


  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;
  // 分页状态 - 直接从 URL 参数派生，无需 useState
  const pagination: LimitType = {
    pos: filterParam.pos || null,
    limit: currentLimit,
    forward: filterParam.forward ?? true,
    more: true,
  };

  // 搜索导航函数
  const searchGo = useSearchNavigate(navigate, filterParam);

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useLimitCountNum(filters);

  // 构建查询参数
  const queryParams: SystemUserChangeLogsParamType = {
    limit: {
      pos: pagination.pos,
      limit: pagination.limit,
      forward: pagination.forward,
      more: pagination.more,
    },
    count_num: countNumManager.getCountNum(),
    ...(filters.log_type && { log_type: filters.log_type }),
    ...(filters.add_user_id !== undefined &&
      filters.add_user_id !== null && { add_user_id: filters.add_user_id }),
  };

  // 获取变更日志数据
  const {
    data: logData,
    isSuccess,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: ["systemUserChangeLogs", queryParams],
    queryFn: ({ signal }) => systemUserChangeLogs(queryParams, { signal }),
  });

  // 处理 Limit 分页查询结果（自动提取 total 和 next）
  isSuccess && countNumManager.handleQueryResult(logData);

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["systemUserChangeLogs"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["systemUserChangeLogs"] });
  };

  const filterForm = useFilterBarForm<z.infer<typeof UserChangeLogFilterFormSchema>>({
    defaultValues: {
      log_type: filterParam.log_type,
      add_user_id: filterParam.add_user_id,
    },
    resolver: zodResolver(UserChangeLogFilterFormSchema) as any,
    initValues: {
      log_type: undefined,
      add_user_id: undefined,
    },
    onSubmit: (data) => {
      navigate({ search: { ...data, pos: null, forward: true } as any });
    },
    onReset: () => {
      navigate({ search: { pos: null, limit: currentLimit, forward: true } as any });
    },
  });

  // 获取API响应数据
  const logs = getQueryResponseData<SystemUserChangeLogItemType[]>(logData, []);
  const cursorData = getQueryResponseCursor(logData);

  // 处理查看详情（用 useMemo 因为在 columns useMemo 中使用）
  const handleViewDetail = useMemo(
    () => (log: SystemUserChangeLogItemType) => {
      setDetailDialog({ open: true, log });
    },
    [],
  );

  // 定义表格列
  const columns = useMemo<ColumnDef<SystemUserChangeLogItemType>[]>(
    () => [
      {
        accessorKey: "id",
        header: () => (
          <div className={cn(isMobile ? "" : "text-right")}>ID</div>
        ),
        size: 80,
        cell: ({ getValue }) => (
          <div
            className={cn("font-mono text-xs", isMobile ? "" : "text-right")}
          >
            {getValue<number>()}
          </div>
        ),
      },
      {
        accessorKey: "log_type",
        header: "日志类型",
        cell: ({ getValue }) => {
          const logType = getValue<string>();
          const label = dictData.change_type?.getLabel(logType) || logType;
          return <Badge variant="secondary">{label}</Badge>;
        },
      },
      {
        accessorKey: "user_data",
        header: "操作用户",
        size: 120,
        cell: ({ getValue }) => (
          <UserDataTooltip userData={getValue() as any} className="text-xs" />
        ),
      },
      {
        accessorKey: "message",
        header: "操作描述",
        size: 350,
        cell: ({ getValue }) => {
          const message = getValue<string>();
          return (
            <div className="max-w-xs truncate text-sm" title={message}>
              {message}
            </div>
          );
        },
      },
      {
        accessorKey: "add_time",
        header: "操作时间",
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
          const log = row.original;
          return (
            <DataTableAction
              className={cn(isMobile ? "justify-end" : "justify-center")}
            >
              <DataTableActionItem
                mobileDisplay="display"
                desktopDisplay="display"
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn("h-7 px-2")}
                  onClick={() => handleViewDetail(log)}
                  title="查看详情"
                >
                  <Eye className={cn("h-4 w-4")} />
                  {isMobile && <span className="ml-2">查看详情</span>}
                </Button>
              </DataTableActionItem>
            </DataTableAction>
          );
        },
      },
    ],
    [handleViewDetail, isMobile, dictData],
  );

  // 关闭详情对话框
  const handleCloseDetailDialog = () => {
    setDetailDialog({ open: false, log: null });
  };

  // isLoading already destructured

  return (
    <div className="container mx-auto p-4  max-w-[1600px] flex flex-col min-h-0 space-y-5">
      {/* 搜索和过滤 */}
      <div className="flex-shrink-0 mb-1 sm:mb-4">
        <FilterBar form={filterForm} className={cn("bg-card rounded-lg border shadow-sm relative")}>
          <FilterBar.Summary>
            <FilterTotalCount value={formatTotalCount(countNumManager.getTotalInfo())} loading={isLoading} />
          </FilterBar.Summary>
          <FilterBar.MobileExtra>
            <ExportMobileButton activeCount={exportAction.activeCount} isLoading={exportAction.activeCount > 0} onClick={exportAction.openDrawer} />
          </FilterBar.MobileExtra>
          {/* 日志类型过滤 */}
          {dictData.change_type && (
            <div className="flex-1 min-w-[180px] max-w-[300px]">
              <FilterDictSelect name="log_type" placeholder="选择日志类型" label="日志类型" disabled={isLoading} dictData={dictData.change_type} allLabel="全部" />
            </div>
          )}
          {/* 操作用户ID过滤 */}
          <div className="flex-1 min-w-[180px] max-w-[300px]">
            <FilterInput name="add_user_id" placeholder="输入用户ID" type="number" label="操作用户ID" disabled={isLoading} />
          </div>
          {/* 动作按钮区域 */}
          <div className={cn("flex-shrink-0")}>
            <FilterActions>
              <FilterSearchButton loading={isLoading} onRefreshSearch={clearCacheAndReload} />
              <FilterResetButton loading={isLoading} />
              <FilterBar.DesktopOnly>
                <ExportSplitButton activeCount={exportAction.activeCount} onSubmitExport={exportAction.submit}
                  onViewHistory={exportAction.openDrawer} isSubmitting={exportAction.isSubmitting} />
              </FilterBar.DesktopOnly>
            </FilterActions>
          </div>
          <FilterBar.MobileFooter>
            {(closeDrawer) => (
              <ExportButton isSubmitting={exportAction.isSubmitting}
                onSubmitExport={() => void exportAction.submit().then(closeDrawer).catch(() => {})} />
            )}
          </FilterBar.MobileFooter>
        </FilterBar>
      </div>

      {/* 表格和分页容器 */}
      <div className="flex-1 flex flex-col overflow-hidden min-h-0">
        {/* 数据表格 */}
        <div className="flex-1 min-h-0">
          <DataTable
            data={logs}
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
            className={cn(
              "h-full [&_.data-table-row]:h-12 [&_td]:py-2 [&_th]:py-2 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b [&_.data-table-wrapper]:overflow-auto [&_.data-table-wrapper]:h-full",
            )}
          />
        </div>

        {/* 分页控件 */}
        <div className="flex-shrink-0 pt-4">
          {countNumManager.hasTotalInfo() && (
            <CursorPagination
              limit={currentLimit}
              cursorData={cursorData}
              searchGo={searchGo}
              totalInfo={countNumManager.getTotalInfo()}
              currentPageSize={logs.length}
              loading={isLoading}
              onRefresh={refreshData}
              showRefresh={true}
              showPageSize={true}
              pageSizeOptions={PAGE_SIZE_OPTIONS}
              onPageSizeChange={(pageSize) => {
                searchGo({
                  limit: pageSize,
                  pos: null,
                  forward: true,
                });
              }}
            />
          )}
        </div>
      </div>

      {detailDialog.log && (
        <ChangeLogDetailDrawer
          log={detailDialog.log}
          open={detailDialog.open}
          onOpenChange={handleCloseDetailDialog}
        />
      )}

      {/* 导出历史抽屉 */}
      <ExportDrawer
        open={exportAction.drawerOpen}
        onOpenChange={(open) => open ? exportAction.openDrawer() : exportAction.closeDrawer()}
        statusDict={dictData.export_task_status!}
        tasks={exportAction.tasks} totalCount={exportAction.totalCount}
        currentPage={exportAction.currentPage} totalPages={exportAction.totalPages}
        onPageChange={exportAction.setPage} isLoading={exportAction.isLoadingTasks}
      />
    </div>
  );
}
