import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { UserDataTooltip } from "@apps/main/components/local/user-data-tooltip";
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import {
  DEFAULT_PAGE_SIZE,
  PagePagination,
  useCountNumManager,
} from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/sub-app/list";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  appSubAppList,
  AppSubAppListItemType,
} from "@shared/apis/user/app";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom//table";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
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
import { Eye, List, Settings } from "lucide-react";
import { useState } from "react";
import { subAppModuleConfig } from "../nav-info";
import { ListNotifyDrawer } from "./list-notify-drawer";
import {
  SubAppListFilterFormSchema
} from "./list-schema";
import { SubAppDetailDrawer } from "./list-sub-app-detail-drawer";
import { SubAppSecretDrawer } from "./list-sub-app-secret-drawer";

export function SubAppListPage() {
  //docs\api\user\app\sub_app_list.md
  //docs\api\user\app\sub_app_secret_view.md
  // 字典数据获取 - 统一在最顶层获取一次

  const { appId } = Route.useParams()
  const [notifyDrawerOpen, setNotifyDrawerOpen] = useState(false)


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
    <>
      <AppDetailNavContainer {...subAppModuleConfig}
        actions={
          <Button variant="outline" size="sm" onClick={() => setNotifyDrawerOpen(true)}>
            <Settings className={cn("mr-2 h-4 w-4")} />
            通知配置
          </Button>
        }>
        <SubAppListContent dictData={dictData} />
      </AppDetailNavContainer>
      <ListNotifyDrawer
        appId={String(appId)}
        open={notifyDrawerOpen}
        onOpenChange={setNotifyDrawerOpen}
      />
    </>
  )
}

// 内容组件：负责内容加载和渲染
interface SubAppListContentProps {

  dictData: TypedDictData<["user_app"]>;
}

function SubAppListContent({ dictData }: SubAppListContentProps) {
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
    sub_app_id: filterParam.sub_app_id ?? null,
    status: filterParam.status ?? null,
  };

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  // 获取子应用列表数据
  const { data: subAppData, isSuccess, isLoading, isError, error } = useQuery({
    queryKey: [
      "appSubAppList",
      appId,
      filterParam.sub_app_id,
      filterParam.status,
      currentPage,
      currentLimit,
    ],
    queryFn: async ({ signal }) => {
      const result = await appSubAppList(
        {
          app_id: Number(appId),
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
  isSuccess && countNumManager.handlePageQueryResult(subAppData);

  // 从查询结果中提取数据
  const subApps = getQueryResponseData<AppSubAppListItemType[]>(subAppData, []);

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["appSubAppList"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["appSubAppList"] });
  };

  // 字典数据已加载，创建状态映射器
  const appStatus = createStatusMapper(
    {
      0: "neutral",
      1: "warning",
      2: "success",
      3: "danger",
    },
    (status) => dictData.app_status?.getLabel(String(status)) || String(status),
  );

  // 定义表格列配置
  const columns: ColumnDef<AppSubAppListItemType>[] = [
    {
      accessorKey: "id",
      header: () => <div className={cn(isMobile ? "" : "text-right")}>ID</div>,
      size: 80,
      cell: ({ getValue }) => (
        <div className={cn("font-mono text-sm", isMobile ? "" : "text-right")}>{getValue<number>()}</div>
      ),
    },
    {
      accessorKey: "name",
      header: "应用名称",
      cell: ({ getValue }) => (
        <div className="font-medium">{getValue<string>()}</div>
      ),
    },
    {
      accessorKey: "client_id",
      header: "应用标识",
      cell: ({ getValue }) => (
        <div className="font-mono text-sm text-muted-foreground">
          {getValue<string>()}
        </div>
      ),
    },
    {
      accessorKey: "status",
      header: "状态",
      size: 100,
      cell: ({ getValue }) => {
        const status = getValue<number>();
        return (
          <Badge className={appStatus.getClass(status)}>
            {appStatus.getText(status)}
          </Badge>
        );
      },
    },
    {
      accessorKey: "sub_req_pending_count",
      header: "待处理请求",
      size: 100,
      cell: ({ getValue }) => {
        const sub_req_pending_count = getValue<number>();
        if (sub_req_pending_count === 0) {
          return <div className="text-sm text-muted-foreground">-</div>;
        }
        return (
          <Link to="/user/app/$appId/request" params={{ appId }}>
            <Button
              variant="ghost"
              size="sm"
              className={cn("h-7 px-2")}
              title="请求列表"
            >
              <List className=" h-4 w-4" />
              <span className="ml-2">待处理 ({sub_req_pending_count || 0}) 个</span>
            </Button>
          </Link>
        );
      },
    },

    {
      accessorKey: "user_data",
      header: "创建用户",
      size: 120,
      cell: ({ getValue }) => (
        <UserDataTooltip userData={getValue() as any} className="text-xs" />
      ),
    },
    {
      accessorKey: "change_time",
      header: "修改时间",
      size: 180,
      cell: ({ getValue }) => (
        <div className="text-sm text-muted-foreground">
          {formatTime(getValue<number>(), TIME_STYLE.ABSOLUTE_TEXT)}
        </div>
      ),
    },
    {
      id: "actions",
      header: () => <div className="text-center">详细</div>,
      size: 80,
      cell: ({ row }) => {
        const subApp = row.original;

        return (
          <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <SubAppDetailDrawer subApp={subApp} appStatusMapper={appStatus}>
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn("h-7 px-2")}
                  title="应用秘钥"
                >
                  <Eye className="h-4 w-4" />
                  <span className="ml-2">查看详情</span>
                </Button>
              </SubAppDetailDrawer>

            </DataTableActionItem>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <SubAppSecretDrawer subApp={subApp} >
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn("h-7 px-2")}
                  title="应用秘钥"
                >
                  <Eye className="h-4 w-4" />
                  <span className="ml-2">应用秘钥</span>
                </Button>
              </SubAppSecretDrawer>
            </DataTableActionItem>
          </DataTableAction>
        );
      },
    },
  ];

  return (
    // 使用与 feature-mail 一致的布局样式和间距
    <div className="flex flex-col min-h-0 space-y-3">
      <div className="flex-shrink-0 mb-1 sm:mb-4">
        {/* 过滤器 */}
        <FilterContainer
          defaultValues={{
            sub_app_id: filterParam.sub_app_id?.toString(),
            status: filterParam.status?.toString(),
          }}
          resolver={zodResolver(SubAppListFilterFormSchema) as any}
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
            <FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />
          }
          className="bg-card rounded-lg border shadow-sm relative"
        >
          {(layoutParams, form) => (
            <div className="flex-1 flex flex-wrap items-end gap-3">
              {/* 子应用ID过滤 */}
              <FilterInput
                name="sub_app_id"
                placeholder="输入应用ID"
                label="应用ID"
                type="number"
                disabled={isLoading}
                layoutParams={layoutParams}
              />

              {/* 状态过滤 */}
              {dictData.app_status && (
                <FilterDictSelect
                  name="status"
                  placeholder="选择状态"
                  label="状态"
                  disabled={isLoading}
                  dictData={dictData.app_status}
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
            data={subApps}
            columns={columns}
            loading={isLoading}
            error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}

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
    </div>
  );
}

// 导出 schema 供路由使用
export { SubAppListFilterParamSchema } from './list-schema';

