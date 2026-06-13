import { FilterBar } from "@apps/main/components/filter-bar/container";
import { FilterActions } from "@apps/main/components/filter-bar/filter-actions/filter-actions";
import { FilterResetButton } from "@apps/main/components/filter-bar/filter-actions/filter-reset-button";
import { FilterSearchButton } from "@apps/main/components/filter-bar/filter-actions/filter-search-button";
import { FilterDictSelect, FilterInput, FilterSystemAppSelector, FilterTotalCount } from "@apps/main/components/filter-bar/filter-fields";
import { ExportButton, ExportMobileButton, ExportSplitButton } from "@apps/main/components/export-manager/export-buttons";
import { ExportDrawer } from "@apps/main/components/export-manager/export-drawer";
import { useAdminExportAction } from "@apps/main/hooks/use-admin-export-action";
import { useFilterBarForm } from "@apps/main/hooks/use-filter-bar-form";
import * as z from "zod";
import { UserDataTooltip } from "@apps/main/components/local/user-data-tooltip";
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
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/admin/user/login-log";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  systemUserLoginHistory,
  type SystemUserLoginHistoryItemType,
  type SystemUserLoginHistoryParamType,
} from "@shared/apis/admin/user";
import { EXPORT_TYPE_SYSTEM_USER_ACCESS } from "@shared/apis/admin/export";
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
import { useCallback, useMemo, useState } from "react";
import { LoginLogDetailDrawer } from "./login-log-detail-drawer";
import { LoginLogFilterFormSchema } from "./login-log-schema";

export function LoginLogPage() {
  //system\user\login_history.md
  // system\user\mapping.md
  //system\user\user_logout.md

  // 字典数据获取 - 统一在最顶层获取一次
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["admin_user", "auth_login", "admin_export"] as const);

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
  return <LoginLogContent dictData={dictData} />;
}

// 内容组件：负责内容加载和渲染
interface LoginLogContentProps {
  dictData: TypedDictData<["admin_user", "auth_login", "admin_export"]>;
}

function LoginLogContent({ dictData }: LoginLogContentProps) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();

  // 获取 URL search 参数
  const filterParam = Route.useSearch();

  // 详情对话框状态
  const [detailDialog, setDetailDialog] = useState({
    open: false,
    login: null as SystemUserLoginHistoryItemType | null,
  });

  // 过滤条件从 URL 参数获取
  const filters = {
    app_id: filterParam.app_id ?? 0,
    oauth_app_id: filterParam.oauth_app_id || null,
    user_id: filterParam.user_id || null,
    is_enable: filterParam.is_enable ?? null,
  };

  // 导出操作 hook
  const exportAction = useAdminExportAction({
    exportType: EXPORT_TYPE_SYSTEM_USER_ACCESS,
    params: {
      app_id: filters.app_id,
      oauth_app_id: filters.oauth_app_id ?? undefined,
      user_id: filters.user_id ?? undefined,
      is_enable: filters.is_enable ?? undefined,
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

  // 构建查询参数， app_id 默认为 0（系统）
  const queryParams: SystemUserLoginHistoryParamType = {
    limit: {
      pos: pagination.pos,
      limit: pagination.limit,
      forward: pagination.forward,
      more: pagination.more,
    },
    count_num: countNumManager.getCountNum(),
    is_enable: filters.is_enable,
    app_id: filters.app_id,
    ...(filters.oauth_app_id !== undefined &&
      filters.oauth_app_id !== null && { oauth_app_id: filters.oauth_app_id }),
    ...(filters.user_id !== undefined &&
      filters.user_id !== null && { user_id: filters.user_id }),
  };

  // 获取登录历史数据
  const {
    data: loginData,
    isSuccess: loginIsSuccess,
    isLoading: loginIsLoading,
    isError,
    error,
  } = useQuery({
    queryKey: ["systemUserLoginHistory", queryParams],
    queryFn: ({ signal }) => systemUserLoginHistory(queryParams, { signal }),
  });

  // 处理 Limit 分页查询结果（自动提取 total 和 next）
  loginIsSuccess && countNumManager.handleQueryResult(loginData);

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["systemUserLoginHistory"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["systemUserLoginHistory"] });
  };

  const filterForm = useFilterBarForm<z.infer<typeof LoginLogFilterFormSchema>>({
    defaultValues: {
      app_id: filterParam.app_id ?? 0,
      oauth_app_id: filterParam.oauth_app_id,
      user_id: filterParam.user_id,
      is_enable: filterParam.is_enable ?? undefined,
    },
    resolver: zodResolver(LoginLogFilterFormSchema) as any,
    initValues: {
      app_id: undefined,
      oauth_app_id: undefined,
      user_id: undefined,
      is_enable: undefined,
    },
    onSubmit: (data) => {
      navigate({ search: { ...data, pos: null, forward: true } as any });
    },
    onReset: () => {
      navigate({ search: { pos: null, limit: currentLimit, forward: true, app_id: 0 } as any });
    },
  });

  const logins = getQueryResponseData<SystemUserLoginHistoryItemType[]>(
    loginData,
    [],
  );
  const cursorData = getQueryResponseCursor(loginData);

  // 状态样式映射
  const statusMapper = useMemo(
    () =>
      createStatusMapper(
        { 1: "success", 2: "danger" },
        (status) =>
          dictData.session_status.getLabel(String(status)) || String(status),
      ),
    [dictData],
  );

  // 处理查看详情
  const handleViewDetail = useCallback(
    (login: SystemUserLoginHistoryItemType) => {
      setDetailDialog({ open: true, login });
    },
    [],
  );

  // 定义表格列
  const columns = useMemo<ColumnDef<SystemUserLoginHistoryItemType>[]>(
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
        accessorKey: "login_type",
        header: "登录类型",
        size: 100,
        cell: ({ getValue }) => (
          <Badge variant="outline">{getValue<string>()}</Badge>
        ),
      },
      {
        accessorKey: "user_data",
        header: "用户信息",
        size: 120,
        cell: ({ getValue }) => (
          <UserDataTooltip userData={getValue() as any} className="text-xs" />
        ),
      },

      {
        accessorKey: "status",
        header: "状态",
        size: 80,
        cell: ({ getValue }) => {
          const status = getValue<number>();
          return (
            <Badge
              variant="secondary"
              className={statusMapper.getClass(status)}
            >
              {statusMapper.getText(status)}
            </Badge>
          );
        },
      },
      {
        accessorKey: "add_time",
        header: "登录时间",
        size: 140,
        cell: ({ getValue }) => {
          const date = getValue<Date>();
          const timeElement = formatTime(date, TIME_STYLE.ABSOLUTE_ELEMENT);
          return (
            <div className="text-xs text-muted-foreground">{timeElement}</div>
          );
        },
      },
      {
        accessorKey: "expire_time",
        header: "过期时间",
        size: 140,
        cell: ({ getValue }) => {
          const date = getValue<Date>();
          const now = new Date();
          const isExpired = date < now;
          return (
            <div
              className={`text-xs ${isExpired ? "text-red-600" : "text-muted-foreground"}`}
            >
              {formatTime(date, TIME_STYLE.ABSOLUTE_ELEMENT)}
            </div>
          );
        },
      },
      {
        id: "actions",
        header: () => <div className="text-center">操作</div>,
        size: 60,
        cell: ({ row }) => {
          const login = row.original;

          return (
            <DataTableAction
              className={cn(isMobile ? "justify-end" : "justify-center")}
            >
              <DataTableActionItem
                mobileDisplay="display"
                desktopDisplay="display"
              >
                <Button
                  onClick={() => handleViewDetail(login)}
                  variant="ghost"
                  size="sm"
                  className={cn("px-2")}
                >
                  <Eye className={cn("h-4 w-4")} />
                  {isMobile && <span className="ml-2">详情</span>}
                </Button>
              </DataTableActionItem>
            </DataTableAction>
          );
        },
      },
    ],
    [handleViewDetail, isMobile, statusMapper],
  );

  // 关闭详情对话框
  const handleCloseDetailDialog = () => {
    setDetailDialog({ open: false, login: null });
  };

  const isLoading = loginIsLoading;

  return (
    <>
      <div className="container mx-auto  p-4 max-w-[1600px] flex flex-col min-h-0 space-y-5">
        {/* 搜索和过滤 */}
        <div className="flex-shrink-0 mb-1 sm:mb-4">
          <FilterBar form={filterForm} className={cn("bg-card rounded-lg border shadow-sm relative")}>
            <FilterBar.Summary>
              <FilterTotalCount value={formatTotalCount(countNumManager.getTotalInfo())} loading={isLoading} />
            </FilterBar.Summary>
            <FilterBar.MobileExtra>
              <ExportMobileButton activeCount={exportAction.activeCount} isLoading={exportAction.activeCount > 0} onClick={exportAction.openDrawer} />
            </FilterBar.MobileExtra>
            {/* 应用选择器 */}
            <div className="flex-1 min-w-[160px] max-w-[240px]">
              <FilterSystemAppSelector name="app_id" label="应用" placeholder="选择应用..." disabled={isLoading} appSelectorProps={{ showStatus: true, showUserInfo: false }} />
            </div>
            {/* OAuth应用选择器 */}
            <div className="flex-1 min-w-[160px] max-w-[240px]">
              <FilterSystemAppSelector name="oauth_app_id" label="OAuth应用" placeholder="选择OAuth应用..." disabled={isLoading} appSelectorProps={{ showStatus: true, showUserInfo: false }} />
            </div>
            {/* 用户ID过滤 */}
            <div className="flex-1 min-w-[130px] max-w-[180px]">
              <FilterInput name="user_id" placeholder="输入用户ID" type="number" label="用户ID" disabled={isLoading} />
            </div>
            {/* 启用状态过滤 */}
            {dictData.session_status && (
              <div className="flex-1 min-w-[140px] max-w-[200px]">
                <FilterDictSelect name="is_enable" placeholder="选择状态" label="会话状态" disabled={isLoading} dictData={dictData.session_status} allLabel="全部" />
              </div>
            )}
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
              data={logins}
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
                currentPageSize={logins.length}
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
        {detailDialog.login && (
          <LoginLogDetailDrawer
            login={detailDialog.login}
            open={detailDialog.open}
            onOpenChange={handleCloseDetailDialog}
            loginStatusDict={dictData.login_status}
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
    </>
  );
}
