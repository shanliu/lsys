import { FilterBar } from "@apps/main/components/filter-bar/container";
import { FilterActions } from "@apps/main/components/filter-bar/filter-actions/filter-actions";
import { FilterSearchButton } from "@apps/main/components/filter-bar/filter-actions/filter-search-button";
import { FilterResetButton } from "@apps/main/components/filter-bar/filter-actions/filter-reset-button";
import { FilterDictSelect, FilterInput, FilterTotalCount } from "@apps/main/components/filter-bar/filter-fields";
import { useFilterBarForm } from "@apps/main/hooks/use-filter-bar-form";
import { formatTotalCount } from "@shared/lib/utils/format-utils";
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import { AppRequestDataDisplay } from "@apps/main/features/user/components/ui/app-request-data-display";
import {
  useDictData,
  type TypedDictData,
} from "@apps/main/hooks/use-dict-data";
import {
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  PagePagination,
  usePageCountNum,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/request";
import { zodResolver } from "@hookform/resolvers/zod";
import { appRequestList, type AppRequestItemType } from "@shared/apis/user/app";
import {
  DataTable,
  DataTableAction,
  DataTableActionItem,
} from "@shared/components/custom//table";
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
import { useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import { Eye } from "lucide-react";
import { useState } from "react";
import { requestModuleConfig } from "./nav-info";
import { RequestDetailDrawer } from "./request-detail-drawer";
import { AppRequestListFilterFormSchema } from "./request-schema";
import * as z from "zod";

export function AppRequestPage() {
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
        className={cn("md:m-6")}
      />
    );
  }

  // 如果字典加载中，显示骨架屏
  if (dictIsLoading) {
    return <PageSkeletonTable variant="page" className={cn("md:m-6")} />;
  }

  // 字典加载成功，渲染内容组件
  return (
    <AppDetailNavContainer {...requestModuleConfig}>
      <AppRequestContent dictData={dictData} />
    </AppDetailNavContainer>
  );
}

// 内容组件：负责内容加载和渲染
interface AppRequestContentProps {
  dictData: TypedDictData<["user_app"]>;
}

function AppRequestContent({ dictData }: AppRequestContentProps) {
  //docs\api\user\app\request_list.md
  const { appId } = Route.useParams();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();

  // 抽屉状态
  const [detailDrawerOpen, setDetailDrawerOpen] = useState(false);
  const [selectedRequest, setSelectedRequest] =
    useState<AppRequestItemType | null>(null);

  // 从 URL 搜索参数获取过滤条件
  const filterParam = Route.useSearch();

  const currentPage = filterParam.page || 1;
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 过滤条件从 URL 参数获取
  const filters = {
    id: filterParam.id || null,
    status: filterParam.status || null,
    request_type: filterParam.request_type || null,
  };

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = usePageCountNum(filters);

  // 获取请求列表数据
  const {
    data: requestData,
    isSuccess,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: [
      "appRequestList",
      appId,
      currentPage,
      currentLimit,
      filters.id,
      filters.status,
      filters.request_type,
    ],
    queryFn: async ({ signal }) => {
      const result = await appRequestList(
        {
          app_id: Number(appId),
          page: { page: currentPage, limit: currentLimit },
          count_num: countNumManager.getCountNum(),
          id: filters.id || undefined,
          status: filters.status || undefined,
          request_type: filters.request_type || undefined,
        },
        { signal },
      );

      // 如果 API 返回失败状态（包括 Zod 验证失败），抛出错误
      if (!result.status) {
        throw new Error(result.message || "数据加载失败");
      }

      return result;
    },
    placeholderData: (previousData) => previousData,
    staleTime: 5000, // 5秒内数据视为新鲜，不会重新请求
    gcTime: 5000, // 5秒后清除缓存
  });

  // 处理 Page 分页查询结果（自动提取 total）
  isSuccess && countNumManager.handleQueryResult(requestData);

  // 获取请求列表数据
  const requests = getQueryResponseData<AppRequestItemType[]>(requestData, []);

  // 搜索导航函数
  const searchGo = useSearchNavigate(navigate, filterParam);

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["appRequestList"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["appRequestList"] });
  };

  const filterForm = useFilterBarForm<z.infer<typeof AppRequestListFilterFormSchema>>({
    defaultValues: {
      id: filterParam.id,
      status: filterParam.status,
      request_type: filterParam.request_type,
    },
    resolver: zodResolver(AppRequestListFilterFormSchema) as any,
    initValues: { id: undefined, status: undefined, request_type: undefined },
    onSubmit: (data) => {
      const d = data as { id?: number; status?: number; request_type?: number };
      searchGo({ id: d.id, status: d.status, request_type: d.request_type, page: 1 });
    },
    onReset: () => {
      searchGo({ page: 1, limit: currentLimit, id: undefined, status: undefined, request_type: undefined });
    },
  });

  // 字典数据已加载，创建状态映射器
  const requestStatusMapper = createStatusMapper(
    {
      1: "warning", // 待审 - 黄色警告色
      2: "success", // 批准 - 绿色成功色
      3: "danger", // 驳回 - 红色危险色
      4: "neutral", // 作废 - 灰色次要色
    },
    (status) =>
      dictData.request_status.getLabel(String(status)) || String(status),
  );

  const requestTypeMapper = (status: number) =>
    dictData.request_type.getLabel(String(status)) || String(status);

  // 打开详情抽屉
  const handleOpenDetail = (request: AppRequestItemType) => {
    setSelectedRequest(request);
    setDetailDrawerOpen(true);
  };

  // 定义表格列配置
  const columns: ColumnDef<AppRequestItemType>[] = [
    {
      accessorKey: "id",
      header: () => <div className={cn(!isMobile && "text-right")}>ID</div>,
      cell: ({ getValue }) => {
        const value = getValue<number>();
        return <div className={cn(!isMobile && "text-right")}>{value}</div>;
      },
      size: 80,
    },
    {
      accessorKey: "request_type",
      header: "请求类型",
      cell: ({ getValue }) => {
        const type = getValue<number>();
        return <Badge variant="outline">{requestTypeMapper(type)}</Badge>;
      },
      size: 140,
    },
    {
      accessorKey: "status",
      header: "状态",
      cell: ({ getValue }) => {
        const status = getValue<number>();
        return (
          <Badge className={requestStatusMapper.getClass(status)}>
            {requestStatusMapper.getText(status)}
          </Badge>
        );
      },
      size: 100,
    },
    {
      accessorKey: "request_data",
      header: "请求数据",
      cell: ({ row }) => {
        return <AppRequestDataDisplay data={row.original} compact />;
      },
      size: 300,
    },
    {
      accessorKey: "request_time",
      header: "请求时间",
      cell: ({ getValue }) => {
        const requestTime = getValue<Date | null>();
        return requestTime
          ? formatTime(requestTime, TIME_STYLE.RELATIVE_ELEMENT)
          : "-";
      },
      size: 120,
    },
    {
      accessorKey: "confirm_time",
      header: "审核时间",
      cell: ({ getValue }) => {
        const confirmTime = getValue<Date | null>();
        return confirmTime
          ? formatTime(confirmTime, TIME_STYLE.ABSOLUTE_TEXT)
          : "-";
      },
      size: 180,
    },
    {
      id: "actions",
      header: () => <div className="text-center">详细</div>,
      cell: ({ row }) => (
        <DataTableAction
          className={cn(isMobile ? "justify-end" : "justify-center")}
        >
          <DataTableActionItem mobileDisplay="display" desktopDisplay="display">
            <Button
              size="sm"
              variant="ghost"
              className={cn("h-7 px-2")}
              onClick={() => handleOpenDetail(row.original)}
              title="查看详情"
            >
              <Eye className="h-4 w-4" />
              {isMobile && <span className="ml-2">查看详情</span>}
            </Button>
          </DataTableActionItem>
        </DataTableAction>
      ),
      size: 80,
    },
  ];

  return (
    <div className="flex flex-col min-h-0 space-y-3">
      <div className="flex-shrink-0 mb-1 sm:mb-4">
        <FilterBar form={filterForm} className="bg-card rounded-lg border shadow-sm relative">
          <FilterBar.Summary>
            <FilterTotalCount value={formatTotalCount(countNumManager.getTotal())} loading={isLoading} />
          </FilterBar.Summary>
          <FilterInput name="id" placeholder="输入请求ID" label="请求ID" disabled={isLoading} type="number" />
          {dictData.request_status && (
            <FilterDictSelect name="status" placeholder="选择状态" label="状态" disabled={isLoading}
              dictData={dictData.request_status} allLabel="全部" />
          )}
          {dictData.request_type && (
            <FilterDictSelect name="request_type" placeholder="选择请求类型" label="请求类型" disabled={isLoading}
              dictData={dictData.request_type} allLabel="全部" />
          )}
          <FilterActions>
            <FilterSearchButton loading={isLoading} onRefreshSearch={clearCacheAndReload} />
            <FilterResetButton loading={isLoading} />
          </FilterActions>
        </FilterBar>
      </div>

      {/* 表格和分页容器 - 确保不超出页面高度 */}
      <div className="flex-1 flex flex-col min-h-0">
        {/* 数据表格 */}
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
            scrollSnapDelay={300}
            leftStickyColumns={[
              { column: 0, minWidth: "80px", maxWidth: "100px" },
            ]}
            className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
            tableContainerClassName="h-full"
          />
        </div>

        {/* 分页控件 */}
        <div className="flex-shrink-0 pt-4 pb-4">
          <PagePagination
            currentPage={currentPage}
            pageSize={currentLimit}
            total={countNumManager.getTotal() ?? 0}
            loading={isLoading}
            onChange={(page) => {
              searchGo({ page });
            }}
            onPageSizeChange={(pageSize) => {
              searchGo({
                limit: pageSize,
                page: 1,
              });
            }}
            onRefresh={refreshData}
            showRefresh={true}
            showPageSize={true}
            pageSizeOptions={PAGE_SIZE_OPTIONS}
          />
        </div>
      </div>

      {/* 详情抽屉 */}
      <RequestDetailDrawer
        request={selectedRequest}
        isOpen={detailDrawerOpen}
        onOpenChange={setDetailDrawerOpen}
        dictData={dictData}
        requestStatusMapper={requestStatusMapper}
      />
    </div>
  );
}
