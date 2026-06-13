import { FilterBar } from "@apps/main/components/filter-bar/container";
import { FilterActions } from "@apps/main/components/filter-bar/filter-actions/filter-actions";
import { FilterSearchButton } from "@apps/main/components/filter-bar/filter-actions/filter-search-button";
import { FilterResetButton } from "@apps/main/components/filter-bar/filter-actions/filter-reset-button";
import { FilterDictSelect, FilterInput, FilterSelect, FilterTotalCount } from "@apps/main/components/filter-bar/filter-fields";
import { useFilterBarForm } from "@apps/main/hooks/use-filter-bar-form";
import { ExportButton, ExportMobileButton, ExportSplitButton } from "@apps/main/components/export-manager/export-buttons";
import { ExportDrawer } from "@apps/main/components/export-manager/export-drawer";
import { useUserExportAction } from "@apps/main/hooks/use-user-export-action";
import { EXPORT_TYPE_USER_LOGIN_HISTORY } from "@shared/apis/user/file";
import {
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  useLimitCountNum,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import {
  accountLoginHistory,
  type AccountLoginHistoryItemType,
} from "@shared/apis/user/account";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { CursorPagination } from "@shared/components/custom/pagination";
import {
  DataTable,
  DataTableAction,
  DataTableActionItem,
} from "@shared/components/custom/table";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { formatTotalCount } from "@shared/lib/utils/format-utils";

import {
  useDictData,
  type TypedDictData,
} from "@apps/main/hooks/use-dict-data";
import { DictList } from "@shared/types/apis-dict";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/user/account/login-log";
import { zodResolver } from "@hookform/resolvers/zod";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { useIsMobile } from "@shared/hooks/use-mobile";
import {
  cn,
  formatTime,
  getQueryResponseCursor,
  getQueryResponseData,
  TIME_STYLE,
} from "@shared/lib/utils";
import { type LimitType } from "@shared/types/base-schema";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ColumnDef } from "@tanstack/react-table";
import { Eye } from "lucide-react";
import { useMemo, useState } from "react";
import { LoginLogDetailDrawer } from "./login-log-detail-drawer";
import {
  LoginLogFilterFormSchema,
  type LoginLogFilterParamType,
} from "./login-log-schema";
import * as z from "zod";

export function AccountLoginLogPage() {
  // 字典数据获取 - 统一在最顶层获取一次
  const {
    dictData: loginDictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["auth_login", "user_export"] as const);

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
        columns={7} // 假设表格有7列
        rows={5}
        className={cn("m-4 md:m-6")}
      />
    );
  }

  // 字典加载成功，渲染内容组件
  return <AccountLoginLogContent loginDictData={loginDictData} />;
}

// 内容组件：负责内容加载和渲染
interface AccountLoginLogContentProps {
  loginDictData: TypedDictData<["auth_login", "user_export"]>;
}

function AccountLoginLogContent({
  loginDictData,
}: AccountLoginLogContentProps) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();

  // 获取 URL search 参数
  const filterParam: LoginLogFilterParamType = Route.useSearch();
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 详情抽屉状态
  const [detailDrawer, setDetailDrawer] = useState({
    open: false,
    log: null as AccountLoginHistoryItemType | null,
  });

  // 过滤条件从 URL 参数获取
  const filters = {
    login_type: filterParam.login_type || null,
    login_account: filterParam.login_account || null,
    login_ip: filterParam.login_ip || null,
    is_login: filterParam.is_login ?? null,
  };

  const exportAction = useUserExportAction({
    exportType: EXPORT_TYPE_USER_LOGIN_HISTORY,
    params: {
      login_type: filters.login_type,
      login_account: filters.login_account,
      login_ip: filters.login_ip,
      is_login: filters.is_login,
    },
  });


  // 分页状态 - 直接从 URL 参数派生，无需 useState
  const pagination: LimitType = {
    pos: filterParam.pos || null,
    limit: filterParam.limit || DEFAULT_PAGE_SIZE,
    forward: filterParam.forward ?? true,
    more: true,
  };

  // 搜索导航函数
  const searchGo = useSearchNavigate(navigate, filterParam);

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useLimitCountNum(filters);

  // 构建查询参数（不包含 count_num，在 queryFn 中动态获取）
  const queryParams = {
    limit: {
      pos: pagination.pos,
      limit: currentLimit,
      forward: pagination.forward,
      more: pagination.more,
    },
    ...(filters.login_type && { login_type: filters.login_type }),
    ...(filters.login_account && { login_account: filters.login_account }),
    ...(filters.login_ip && { login_ip: filters.login_ip }),
    ...(filters.is_login !== null &&
      filters.is_login !== undefined && { is_login: filters.is_login }),
  };

  // 获取登录日志数据
  const {
    data: logData,
    isSuccess,
    isLoading: queryIsLoading,
    isFetching,
    isError,
    error,
  } = useQuery({
    queryKey: ["accountLoginHistory", queryParams],
    queryFn: ({ signal }) =>
      // 在查询时动态获取 count_num，确保使用最新值
      accountLoginHistory(
        {
          ...queryParams,
          count_num: countNumManager.getCountNum(),
        },
        { signal },
      ),
    placeholderData: (previousData) => previousData, // 保持之前的数据，防止加载时数据清空
  });

  // 处理 Limit 分页查询结果（自动提取 total 和 next）
  isSuccess && countNumManager.handleQueryResult(logData);

  // 获取API响应数据
  const logs = getQueryResponseData<AccountLoginHistoryItemType[]>(logData, []);
  const cursorData = getQueryResponseCursor(logData);

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["accountLoginHistory"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    // 1. 重置 count_num 为 true
    countNumManager.reset();

    // 2. 使其失效并重新获取（保证 count_num: true 会生效）
    queryClient.invalidateQueries({
      queryKey: ["accountLoginHistory"],
    });
  };

  // 预处理字典数据，避免重复判断
  // login_type 含额外 validity 字段，运行时已是 DictList 实例，断言以使用其方法
  const loginTypeDict = loginDictData.login_type as unknown as DictList;

  // 字典数据已加载，创建登录状态映射器
  const loginStatus = createStatusMapper(
    {
      0: "danger", // 失败
      1: "warning", // 仅预登陆
      2: "success", // 成功
    },
    (status) =>
      loginDictData.login_status?.getLabel(String(status)) || String(status),
  );

  // 定义表格列
  const columns = useMemo<ColumnDef<AccountLoginHistoryItemType>[]>(
    () => [
      {
        accessorKey: "id",
        header: () => (
          <div className={cn(isMobile ? "" : "text-right")}>ID</div>
        ),
        size: 80,
        cell: ({ getValue }) => (
          <div
            className={cn(
              "font-mono text-xs whitespace-nowrap",
              isMobile ? "" : "text-right",
            )}
          >
            {getValue<number>()}
          </div>
        ),
      },
      {
        accessorKey: "login_account",
        header: "登录账号",
        cell: ({ getValue }) => (
          <CopyableText value={getValue<string>()} className={cn("text-sm")} />
        ),
      },
      {
        accessorKey: "login_type",
        header: "登录类型",
        size: 120,
        minSize: 50,
        cell: ({ getValue }) => {
          const loginType = getValue<string>();
          const loginTypeLabel = loginTypeDict.getLabel(loginType);
          return loginTypeLabel;
        },
      },
      {
        accessorKey: "login_ip",
        header: "登录IP",
        cell: ({ getValue }) => (
          <CopyableText
            value={getValue<string>()}
            className={cn("text-xs whitespace-nowrap")}
          />
        ),
      },
      {
        accessorKey: "is_login",
        header: "登录结果",
        cell: ({ getValue }) => {
          const isLogin = getValue<number>();
          return (
            <Badge
              className={cn(loginStatus.getClass(isLogin), "whitespace-nowrap")}
            >
              {loginStatus.getText(isLogin)}
            </Badge>
          );
        },
      },
      {
        accessorKey: "add_time",
        header: "登录时间",
        cell: ({ getValue }) => {
          const date = getValue<Date | null>();
          const timeElement = formatTime(date, TIME_STYLE.RELATIVE_ELEMENT);
          return (
            <div className="text-xs text-muted-foreground whitespace-nowrap">
              {timeElement}
            </div>
          );
        },
      },
      {
        id: "actions",
        size: 80,
        header: () => <div className="text-center">详细</div>,
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
                  className={cn("h-7 ")}
                  onClick={() => setDetailDrawer({ open: true, log })}
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
    ],
    [loginTypeDict, loginStatus, isMobile],
  );

  const isLoading = queryIsLoading || isFetching;

  const filterForm = useFilterBarForm<z.infer<typeof LoginLogFilterFormSchema>>({
    defaultValues: {
      login_type: filterParam.login_type,
      login_account: filterParam.login_account,
      login_ip: filterParam.login_ip,
      is_login: filterParam.is_login,
    },
    resolver: zodResolver(LoginLogFilterFormSchema) as any,
    initValues: { login_type: undefined, login_account: undefined, login_ip: undefined, is_login: undefined },
    onSubmit: (data) => {
      if (isError) {
        refreshData();
        return;
      }
      navigate({ search: { ...data, pos: null, forward: true } as any });
    },
    onReset: () => {
      navigate({ search: { pos: null, limit: currentLimit, forward: true } as any });
    },
  });

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-4 py-3 max-w-[1600px] flex flex-col min-h-0 space-y-4">
      {/* 搜索和过滤 */}
      <div className="flex-shrink-0 mb-1 sm:mb-4">
        <FilterBar form={filterForm} className="bg-card rounded-lg border shadow-sm relative">
          <FilterBar.Summary>
            <FilterTotalCount value={formatTotalCount(countNumManager.getTotalInfo())} loading={isLoading} />
          </FilterBar.Summary>
          <FilterBar.MobileExtra>
            <ExportMobileButton activeCount={exportAction.activeCount} isLoading={exportAction.activeCount > 0} onClick={exportAction.openDrawer} />
          </FilterBar.MobileExtra>
          <FilterSelect name="login_type" placeholder="选择登录类型" label="登录类型"
            disabled={isLoading} allLabel="全部类型" options={loginTypeDict.getOptions()} />
          <FilterInput name="login_account" placeholder="输入登录账号" label="登录账号" disabled={isLoading} />
          <FilterInput name="login_ip" placeholder="输入IP地址" label="登录IP" disabled={isLoading} />
          {loginDictData.login_status && (
            <FilterDictSelect name="is_login" placeholder="选择状态" label="登录状态"
              disabled={isLoading} dictData={loginDictData.login_status} allLabel="全部" />
          )}
          <FilterActions>
            <FilterSearchButton loading={isLoading} onRefreshSearch={clearCacheAndReload} />
            <FilterResetButton loading={isLoading} />
            <FilterBar.DesktopOnly>
              <ExportSplitButton activeCount={exportAction.activeCount} onSubmitExport={exportAction.submit}
                onViewHistory={exportAction.openDrawer} isSubmitting={exportAction.isSubmitting} />
            </FilterBar.DesktopOnly>
          </FilterActions>
          <FilterBar.MobileFooter>
            {(closeDrawer) => (
              <ExportButton isSubmitting={exportAction.isSubmitting}
                onSubmitExport={() => void exportAction.submit().then(closeDrawer).catch(() => {})} />
            )}
          </FilterBar.MobileFooter>
        </FilterBar>
        <ExportDrawer
          open={exportAction.drawerOpen}
          onOpenChange={(open) => open ? exportAction.openDrawer() : exportAction.closeDrawer()}
          statusDict={loginDictData.export_task_status!}
          tasks={exportAction.tasks} totalCount={exportAction.totalCount}
          currentPage={exportAction.currentPage} totalPages={exportAction.totalPages}
          onPageChange={exportAction.setPage} isLoading={exportAction.isLoadingTasks}
        />
      </div>

      {/* 表格和分页容器 */}
      <div className="flex-1 flex flex-col overflow-hidden min-h-0">
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

        <div className="flex-shrink-0 pt-2">
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
        </div>

        <LoginLogDetailDrawer
          log={detailDrawer.log}
          open={detailDrawer.open}
          onOpenChange={() => setDetailDrawer({ open: false, log: null })}
          loginDictData={loginDictData}
        />
      </div>
    </div>
  );
}

// 导出 schema 供路由使用
export { LoginLogFilterParamSchema } from "./login-log-schema";
