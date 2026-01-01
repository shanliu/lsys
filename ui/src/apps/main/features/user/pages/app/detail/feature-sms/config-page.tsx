"use client";

import { userSenderSmsConfigDel, UserSenderSmsConfigItemType, userSenderSmsConfigList } from "@shared/apis/user/sender-sms";
import { SenderRuleConfigView } from "@apps/main/components/local/sender-config/rule-config-view";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { DEFAULT_PAGE_SIZE, PagePagination, useCountNumManager } from "@apps/main/lib/pagination-utils";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import { TypedDictData, useDictData } from "@apps/main/hooks/use-dict-data";
import { useIsMobile } from "@shared/hooks/use-mobile";
import { cn, formatServerError, formatTime, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-sms/config";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ColumnDef } from "@tanstack/react-table";
import { Plus, Trash2 } from "lucide-react";
import React from "react";
import { featureSmsModuleConfig } from "../nav-info";
import { SmsConfigDrawer } from "./config-drawer";
import { SmsConfigFilterFormSchema } from "./config-schema";

export default function AppDetailFeatureSmsConfigPage() {
  // user\app_sender_smser\config_add.md
  // user\app_sender_smser\config_del.md
  // user\app_sender_smser\config_list.md
  const { appId } = Route.useParams();
  const [configDrawerOpen, setConfigDrawerOpen] = React.useState(false);

  // 字典数据获取 - 统一在最顶层获取一次
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["user_sender_sms"] as const);

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
      <AppDetailNavContainer
        {...featureSmsModuleConfig}
        actions={
          <Button size="sm" variant="outline" onClick={() => setConfigDrawerOpen(true)}>
            <Plus className={cn("mr-2 h-4 w-4")} />
            新增配置
          </Button>
        }
      >
        <SmsConfigContent dictData={dictData} />
      </AppDetailNavContainer>

      <SmsConfigDrawer
        appId={appId}
        dictData={dictData}
        open={configDrawerOpen}
        onOpenChange={setConfigDrawerOpen}
      />
    </>
  );
}


interface SmsConfigContentProps {
  dictData: TypedDictData<["user_sender_sms"]>;
}

export function SmsConfigContent({ dictData }: SmsConfigContentProps) {
  const { appId } = Route.useParams();
  const toast = useToast();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();

  // 从 URL 搜索参数获取过滤条件
  const filterParam = Route.useSearch();

  const currentPage = filterParam.page || 1;
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 过滤条件
  const filters = {
    config_type: filterParam.config_type || null,
  };

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  // 获取短信配置列表数据
  const {
    data: configData,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: [
      "sms-config-list",
      appId,
      filterParam.config_type,
      currentPage,
      currentLimit,
    ],
    queryFn: async ({ signal }) => {
      const result = await userSenderSmsConfigList(
        {
          app_id: Number(appId),
        },
        { signal }
      );
      return result;
    },
    placeholderData: (previousData) => previousData,
  });

  // 从查询结果中提取数据
  const configs = getQueryResponseData<UserSenderSmsConfigItemType[]>(configData, []);

  // 删除配置
  const deleteMutation = useMutation({
    mutationFn: (id: number) => userSenderSmsConfigDel({ config_id: id }),
    onSuccess: () => {
      toast.success("配置删除成功");
      queryClient.invalidateQueries({ queryKey: ["sms-config-list", appId] });
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
    queryClient.refetchQueries({ queryKey: ["sms-config-list"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["sms-config-list"] });
  };

  // 定义表格列配置
  const columns: ColumnDef<UserSenderSmsConfigItemType>[] = [
    {
      accessorKey: "id",
      header: () => <div className={cn(isMobile ? "" : "text-right")}>ID</div>,
      size: 80,
      cell: ({ getValue }) => (
        <div className={cn("font-mono text-sm", isMobile ? "" : "text-right")}>{getValue<number>()}</div>
      ),
    },
    {
      accessorKey: "config_type",
      header: "配置类型",
      cell: ({ getValue }) => {
        const configType = getValue<string>();
        const typeLabel = dictData.sms_config_type?.getLabel(configType) || configType;
        return <Badge>{typeLabel}</Badge>;
      },
    },
    {
      accessorKey: "priority",
      header: () => <div className={cn(isMobile ? "" : "text-right")}>优先级</div>,
      size: 100,
      cell: ({ getValue }) => (
        <div className={cn("font-mono text-sm", isMobile ? "" : "text-right")}>{getValue<string>()}</div>
      ),
    },
    {
      accessorKey: "config_data",
      header: "配置数据",
      cell: ({ row }) => {
        const config = row.original;
        return (
          <SenderRuleConfigView
            configType={Number(config.config_type)}
            configData={config.config_data}
            displayType="table"
          />
        );
      },
    },
    {
      accessorKey: "add_time",
      header: "添加时间",
      size: 150,
      cell: ({ getValue }) => (
        <div className="text-sm text-muted-foreground whitespace-nowrap">
          {formatTime(getValue<number>(), TIME_STYLE.ABSOLUTE_TEXT)}
        </div>
      ),
    },
    {
      id: "actions",
      header: () => <div className="text-center">操作</div>,
      size: 80,
      cell: ({ row }) => {
        const config = row.original;

        return (
          <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="display">
              <ConfirmDialog
                title="确认删除"
                description={
                  <div>
                    确定要删除配置 <strong>ID: {config.id}</strong> 吗？
                    <br />
                    删除后无法恢复。
                  </div>
                }
                onConfirm={() => handleDelete(Number(config.id))}
              >
                <Button
                  variant="ghost"
                  className={cn("h-7 px-2 ")}
                  title="删除"
                >
                  <Trash2 className=" h-4 w-4" />
                   {isMobile ? <span className="ml-2">删除</span>: null}
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
            config_type: filterParam.config_type,
          }}
          resolver={zodResolver(SmsConfigFilterFormSchema) as any}
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
              total={configs.length ?? 0}
              loading={isLoading}
            />
          }
          className="bg-card rounded-lg border shadow-sm relative"
        >
          {(layoutParams, form) => (
            <div className="flex-1 flex flex-wrap items-end gap-3">
              {/* 配置类型过滤 */}
              {dictData.sms_config_type && (
                <FilterDictSelect
                  name="config_type"
                  placeholder="选择配置类型"
                  label="配置类型"
                  disabled={isLoading}
                  dictData={dictData.sms_config_type}
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
            data={configs}
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
            total={configs.length ?? 0}
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
export { SmsConfigFilterParamSchema } from './config-schema';

