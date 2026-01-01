import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterSystemAppSelector } from "@apps/main/components/filter-container/filter-system-app-selector";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
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
import { Route } from "@apps/main/routes/_main/admin/user/login-log";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  systemUserLoginHistory,
  type SystemUserLoginHistoryItemType,
  type SystemUserLoginHistoryParamType,
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
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ColumnDef } from "@tanstack/react-table";
import { Eye } from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { LoginLogDetailDrawer } from "./login-log-detail-drawer";
import {
  LoginLogFilterFormSchema,
  type LoginOffsetPaginationType
} from "./login-log-schema";

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
  dictData: TypedDictData<["admin_user"]>;
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

  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 分页状态 - 直接从 URL 参数派生，无需 useState
  const pagination: LoginOffsetPaginationType = {
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

  // 构建查询参数， app_id 默认为 0（系统）
  const queryParams: SystemUserLoginHistoryParamType = {
    limit: {
      eq_pos: pagination.eq_pos,
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
  const { data: loginData, isSuccess: loginIsSuccess, isLoading: loginIsLoading, isError, error } = useQuery({
    queryKey: ["systemUserLoginHistory", queryParams],
    queryFn: ({ signal }) => systemUserLoginHistory(queryParams, { signal }),
  });

  // 处理 Limit 分页查询结果（自动提取 total 和 next）
  loginIsSuccess && countNumManager.handleLimitQueryResult(loginData);

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["systemUserLoginHistory"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["systemUserLoginHistory"] });
  };



  const logins = getQueryResponseData<SystemUserLoginHistoryItemType[]>(
    loginData,
    [],
  );
  const nextPageStartPos = getQueryResponseNext(loginData);

  // 状态样式映射
  const statusMapper = useMemo(
    () => createStatusMapper(
      { 1: "success", 2: "danger" },
      (status) => dictData.session_status.getLabel(String(status)) || String(status),
    ),
    [dictData]
  );

  // 处理查看详情
  const handleViewDetail = useCallback((login: SystemUserLoginHistoryItemType) => {
    setDetailDialog({ open: true, login });
  }, []);



  // 定义表格列
  const columns = useMemo<ColumnDef<SystemUserLoginHistoryItemType>[]>(
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
            <Badge variant="secondary" className={statusMapper.getClass(status)}>
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
            <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>

              <DataTableActionItem mobileDisplay="display" desktopDisplay="display">
                <Button onClick={() => handleViewDetail(login)} variant="ghost" size="sm" className={cn("px-2")}>
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

  // 使用统一的分页处理器
  const { handleNextPage, handlePrevPage, canGoNext, canGoPrev } =
    useOffsetPaginationHandlers({
      ...extractMinMax(logins, 'id', 'minId', 'maxId'),
      pagination,
      nextPageStartPos,
      searchGo,
      defaultForward: false, // 从大到小排序（新到旧）
    });

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
          <FilterContainer
            defaultValues={{
              app_id: filterParam.app_id !== undefined ? String(filterParam.app_id) : "0",
              oauth_app_id: filterParam.oauth_app_id?.toString(),
              user_id: filterParam.user_id?.toString(),
              is_enable: filterParam.is_enable?.toString(),
            }}
            resolver={zodResolver(LoginLogFilterFormSchema) as any}
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
                  app_id: 0,
                } as any,
              });
            }}
            countComponent={<FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />}
            className={cn("bg-card rounded-lg border shadow-sm relative")}
          >
            {(layoutParams, form) => (
              <div className="flex-1 flex flex-wrap items-end gap-3 lg:gap-4">
                {/* 应用选择器 */}
                <div className="flex-1 min-w-[160px] max-w-[240px]">
                  <FilterSystemAppSelector
                    name="app_id"
                    label="应用"
                    placeholder="选择应用..."
                    disabled={isLoading}
                    layoutParams={layoutParams}
                    appSelectorProps={{
                      showStatus: true,
                      showUserInfo: false,
                    }}
                  />
                </div>

                {/* OAuth应用选择器 */}
                <div className="flex-1 min-w-[160px] max-w-[240px]">
                  <FilterSystemAppSelector
                    name="oauth_app_id"
                    label="OAuth应用"
                    placeholder="选择OAuth应用..."
                    disabled={isLoading}
                    layoutParams={layoutParams}
                    appSelectorProps={{
                      showStatus: true,
                      showUserInfo: false,
                    }}
                  />
                </div>

                {/* 用户ID过滤 */}
                <div className="flex-1 min-w-[130px] max-w-[180px]">
                  <FilterInput
                    name="user_id"
                    placeholder="输入用户ID"
                    type="number"
                    label="用户ID"
                    disabled={isLoading}
                    layoutParams={layoutParams}
                  />
                </div>

                {/* 启用状态过滤 */}
                {dictData.session_status && (
                  <div className="flex-1 min-w-[140px] max-w-[200px]">
                    <FilterDictSelect
                      name="is_enable"
                      placeholder="选择状态"
                      label="会话状态"
                      disabled={isLoading}
                      dictData={dictData.session_status}
                      layoutParams={layoutParams}
                      allLabel="全部"
                    />
                  </div>
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

        {/* 表格和分页容器 */}
        <div className="flex-1 flex flex-col overflow-hidden min-h-0">
          {/* 数据表格 */}
          <div className="flex-1 min-h-0">
            <DataTable
              data={logins}
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
                limit={pagination.limit}
                hasNext={canGoNext}
                canGoPrev={canGoPrev}
                total={countNumManager.getTotal()}
                currentPageSize={logins.length}
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
        {detailDialog.login && <LoginLogDetailDrawer
          login={detailDialog.login}
          open={detailDialog.open}
          onOpenChange={handleCloseDetailDialog}
        />}
      </div>
    </>
  );
}
