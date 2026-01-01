"use client";

import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import {
  DEFAULT_PAGE_SIZE,
  useCountNumManager,
} from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-barcode/list-parse";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  userBarcodeParseRecordDelete,
  userBarcodeParseRecordList,
  type UserBarcodeParseRecordItemType,
} from "@shared/apis/user/barcode";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { PagePagination } from "@shared/components/custom/pagination";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import { useIsMobile } from "@shared/hooks/use-mobile";
import {
  cn,
  formatServerError,
  formatTime,
  getQueryResponseData,
  TIME_STYLE,
} from "@shared/lib/utils";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import { ExternalLink, Eye, Trash2 } from "lucide-react";
import React from "react";
import { featureBarcodeModuleConfig } from "../nav-info";
import { BarcodeParseRecordDetailDrawer } from "./list-parse-drawer";
import {
  BarcodeParseRecordFilterFormSchema
} from "./list-parse-schema";

export default function AppDetailFeatureBarCodeListParsePage() {
  // user\app_barcode\mapping.md
  // user\app_barcode\parse_record_delete.md
  // user\app_barcode\parse_record_list.md


  // 字典数据获取 - 统一在最顶层获取一次
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["user_barcode"] as const);

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
    <AppDetailNavContainer
      {...featureBarcodeModuleConfig}
      actions={
        <Button
          variant="outline"
          size="sm"
          onClick={() => window.open('https://github.com/shanliu/lsys/blob/dev/docs/rest/barcode/parse.md', '_blank')}
        >
          <ExternalLink className=" h-4 w-4" />
          <span className="ml-2">接入文档</span>
        </Button>
      }
    >
      <BarcodeParseRecordContent dictData={dictData} />
    </AppDetailNavContainer>
  );
}

// 内容组件：负责内容加载和渲染
interface BarcodeParseRecordContentProps {
  dictData: TypedDictData<["user_barcode"]>;
}

function BarcodeParseRecordContent({ dictData }: BarcodeParseRecordContentProps) {
  const { appId } = Route.useParams();
  const toast = useToast();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();
  const [detailDrawerOpen, setDetailDrawerOpen] = React.useState(false);
  const [selectedRecord, setSelectedRecord] = React.useState<UserBarcodeParseRecordItemType | undefined>(undefined);

  // 从 URL 搜索参数获取过滤条件
  const filterParam = Route.useSearch();

  const currentPage = filterParam.page || 1;
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 过滤条件
  const filters = {
    barcode_type: filterParam.barcode_type || null,
    status: filterParam.status || null,
  };

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  // 获取解析记录列表数据
  const {
    data: recordData,
    isSuccess,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: [
      "barcode-parse-record-list",
      appId,
      filterParam.barcode_type,
      filterParam.status,
      currentPage,
      currentLimit,
    ],
    queryFn: async ({ signal }) => {
      const result = await userBarcodeParseRecordList(
        {
          app_id: Number(appId),
          barcode_type: filterParam.barcode_type,
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
  isSuccess && countNumManager.handlePageQueryResult(recordData);

  // 从查询结果中提取数据
  const records = getQueryResponseData<UserBarcodeParseRecordItemType[]>(recordData, []);

  // 删除记录
  const deleteMutation = useMutation({
    mutationFn: (id: number) => userBarcodeParseRecordDelete({ id }),
    onSuccess: () => {
      toast.success("解析记录删除成功");
      queryClient.invalidateQueries({ queryKey: ["barcode-parse-record-list", appId] });
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    },
  });

  const handleDelete = async (id: number) => {
    await deleteMutation.mutateAsync(id);
  };

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["barcode-parse-record-list"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["barcode-parse-record-list"] });
  };

  const parseStatusMapper = createStatusMapper<string>(
    {
      "1": "success",
      "2": "danger",
    },
    (status) => dictData.parse_status.getLabel(String(status)) || String(status)
  );

  // 定义表格列配置
  const columns: ColumnDef<UserBarcodeParseRecordItemType>[] = [
    {
      accessorKey: "id",
      header: () => <div className={cn(isMobile ? "" : "text-right")}>ID</div>,
      size: 80,
      cell: ({ getValue }) => (
        <div className={cn("font-mono text-sm", isMobile ? "" : "text-right")}>{getValue<number>()}</div>
      ),
    },
    {
      accessorKey: "bar_type",
      header: "条码类型",
      cell: ({ getValue }) => {
        const barcodeType = getValue<string>();
        const barcodeTypeLabel = dictData.barcode_type.getLabel(barcodeType) || barcodeType;
        return <div className="font-mono text-sm">{barcodeTypeLabel}</div>;
      },
    },
    {
      accessorKey: "status",
      header: "状态",
      size: 100,
      cell: ({ getValue }) => {
        const status = String(getValue<string | number>() ?? "");
        return (
          <Badge className={parseStatusMapper.getClass(status)}>
            {parseStatusMapper.getText(status)}
          </Badge>
        );
      },
    },
    {
      accessorKey: "error",
      header: "错误信息",
      cell: ({ getValue }) => {
        const error = getValue<string>();
        return error ? (
          <div className="text-sm text-destructive truncate max-w-xs" title={error}>
            {error}
          </div>
        ) : (
          <div className="text-sm text-muted-foreground">-</div>
        );
      },
    },
    {
      accessorKey: "create_time",
      header: "创建时间",
      size: 180,
      cell: ({ getValue }) => (
        <div className="text-sm text-muted-foreground">
          {formatTime(getValue<number>(), TIME_STYLE.ABSOLUTE_TEXT)}
        </div>
      ),
    },
    {
      id: "actions",
      header: () => <div className="text-center">操作</div>,
      size: 80,
      cell: ({ row }) => {
        const record = row.original;

        return (
          <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <Button
                variant="ghost"
                size="sm"
                className={cn("h-7 px-2")}
                onClick={() => {
                  setSelectedRecord(record);
                  setDetailDrawerOpen(true);
                }}
              >
                <Eye className={cn("h-4 w-4", isMobile ? "mr-2" : "")} />
                {isMobile ? "详细" : "查看详细"}
              </Button>
            </DataTableActionItem>

            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <ConfirmDialog
                title="确认删除"
                description={
                  <div>
                    确定要删除解析记录 <strong>ID: {record.id}</strong> 吗？
                    <br />
                    删除后无法恢复。
                  </div>
                }
                onConfirm={() => handleDelete(Number(record.id))}
              >
                <Button
                  size="sm"
                  variant="ghost"
                  className={cn("h-7 px-2")}
                  title="删除"
                >
                  <Trash2 className={cn("h-4 w-4", isMobile ? "mr-2" : "")} />
                  {isMobile ? "删除" : "确认删除"}
                </Button>
              </ConfirmDialog>
            </DataTableActionItem>
          </DataTableAction>
        );
      },
    },
  ];

  return (
    <div className="flex flex-col min-h-0 space-y-3">
      <div className="flex-shrink-0 mb-1 sm:mb-4">
        {/* 过滤器 */}
        <FilterContainer
          defaultValues={{
            barcode_type: filterParam.barcode_type,
            status: filterParam.status,
          }}
          resolver={zodResolver(BarcodeParseRecordFilterFormSchema) as any}
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
              {/* 条码类型过滤 */}
              <FilterDictSelect
                name="barcode_type"
                placeholder="选择条码类型"
                label="条码类型"
                disabled={isLoading}
                dictData={dictData.barcode_type}
                layoutParams={layoutParams}
                allLabel="全部"
              />

              {/* 状态过滤 */}
              {
                <FilterDictSelect
                  name="status"
                  placeholder="选择状态"
                  label="状态"
                  disabled={isLoading}
                  dictData={dictData.parse_status}
                  layoutParams={layoutParams}
                  allLabel="全部"
                />
              }

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
            data={records}
            columns={columns}
            loading={isLoading}
            error={
              isError ? (
                <CenteredError error={error} variant="content" onReset={refreshData} />
              ) : null
            }
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

      {selectedRecord && (
        <BarcodeParseRecordDetailDrawer
          record={selectedRecord}
          dictData={dictData}
          open={detailDrawerOpen}
          onOpenChange={setDetailDrawerOpen}
        />
      )}
    </div>
  );
}

// 导出 schema 供路由使用
export { BarcodeParseRecordFilterParamSchema } from './list-parse-schema';

