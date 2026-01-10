import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { UserDataTooltip } from "@apps/main/components/local/user-data-tooltip";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import {
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  useCountNumManager,
  useOffsetPaginationHandlers,
  useSearchNavigate
} from "@apps/main/lib/pagination-utils";
import { Route } from "@apps/main/routes/_main/admin/user/change-log";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  type SystemUserChangeLogItemType,
  systemUserChangeLogs,
  type SystemUserChangeLogsParamType,
} from "@shared/apis/admin/user";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { OffsetPagination } from "@shared/components/custom/pagination";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
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
import { type LimitType } from "@shared/types/base-schema";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ColumnDef } from "@tanstack/react-table";
import { Eye } from "lucide-react";
import { useMemo, useState } from "react";
import { ChangeLogDetailDrawer } from "./change-log-detail-drawer";
import {
  UserChangeLogFilterFormSchema
} from "./change-log-schema";

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
  } = useDictData(["admin_user"] as const);

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
  dictData: TypedDictData<["admin_user"]>;
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
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;
  // 分页状态 - 直接从 URL 参数派生，无需 useState
  const pagination: LimitType = {
    pos: filterParam.pos || null,
    limit: currentLimit,
    forward: filterParam.forward || false,
    more: true,
    eq_pos: filterParam.eq_pos || false,
  };

  // 搜索导航函数
  const searchGo = useSearchNavigate(navigate, filterParam);

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  // 构建查询参数
  const queryParams: SystemUserChangeLogsParamType = {
    limit: {
      eq_pos: pagination.eq_pos,
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
  const { data: logData, isSuccess, isLoading, isError, error } = useQuery({
    queryKey: ["systemUserChangeLogs", queryParams],
    queryFn: ({ signal }) => systemUserChangeLogs(queryParams, { signal }),
  });

  // 处理 Limit 分页查询结果（自动提取 total 和 next）
  isSuccess && countNumManager.handleLimitQueryResult(logData);

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["systemUserChangeLogs"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["systemUserChangeLogs"] });
  };

  // 获取API响应数据
  const logs = getQueryResponseData<SystemUserChangeLogItemType[]>(logData, []);
  const nextPageStartPos = getQueryResponseNext(logData);

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
        header: () => <div className={cn(isMobile ? "" : "text-right")}>ID</div>,
        size: 80,
        cell: ({ getValue }) => (
          <div className={cn("font-mono text-xs", isMobile ? "" : "text-right")}>{getValue<number>()}</div>
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
            <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
              <DataTableActionItem mobileDisplay="display" desktopDisplay="display">
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

  // 使用统一的分页处理器
  const { handleNextPage, handlePrevPage, canGoNext, canGoPrev } =
    useOffsetPaginationHandlers({
      ...extractMinMax(logs, 'id', 'minId', 'maxId'),
      pagination,
      nextPageStartPos,
      searchGo,
      defaultForward: false, // 从大到小排序（新到旧）
    });

  // 关闭详情对话框
  const handleCloseDetailDialog = () => {
    setDetailDialog({ open: false, log: null });
  };

  // isLoading already destructured

  return (
    <div className="container mx-auto p-4  max-w-[1600px] flex flex-col min-h-0 space-y-5">
      {/* 搜索和过滤 */}
      <div className="flex-shrink-0 mb-1 sm:mb-4">
        <FilterContainer
          defaultValues={{
            log_type: filterParam.log_type,
            add_user_id: filterParam.add_user_id?.toString(),
          }}
          resolver={zodResolver(UserChangeLogFilterFormSchema) as any}
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
                limit: currentLimit,
                forward: false,
                eq_pos: false,
              } as any,
            });
          }}
          countComponent={<FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />}
          className={cn("bg-card rounded-lg border shadow-sm relative")}
        >
          {(layoutParams, form) => (
            <div className="flex-1 flex flex-wrap items-end gap-3 lg:gap-4">
              {/* 日志类型过滤 */}
              {dictData.change_type && (
                <div className="flex-1 min-w-[180px] max-w-[300px]">
                  <FilterDictSelect
                    name="log_type"
                    placeholder="选择日志类型"
                    label="日志类型"
                    disabled={isLoading}
                    dictData={dictData.change_type}
                    layoutParams={layoutParams}
                    allLabel="全部"
                  />
                </div>
              )}

              {/* 操作用户ID过滤 */}
              <div className="flex-1 min-w-[180px] max-w-[300px]">
                <FilterInput
                  name="add_user_id"
                  placeholder="输入用户ID"
                  type="number"
                  label="操作用户ID"
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
            data={logs}
            columns={columns}
            loading={isLoading}
            error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}
            className={cn("h-full [&_.data-table-row]:h-12 [&_td]:py-2 [&_th]:py-2 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b [&_.data-table-wrapper]:overflow-auto [&_.data-table-wrapper]:h-full")}
          />
        </div>

        {/* 分页控件 */}
        <div className="flex-shrink-0 pt-4">
          {(countNumManager.getTotal() ?? 0) > 0 && (
            <OffsetPagination
              limit={currentLimit}
              hasNext={canGoNext}
              canGoPrev={canGoPrev}
              total={countNumManager.getTotal()}
              currentPageSize={logs.length}
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

      {detailDialog.log && <ChangeLogDetailDrawer
        log={detailDialog.log}
        open={detailDialog.open}
        onOpenChange={handleCloseDetailDialog}
      />}
    </div>
  );
}
