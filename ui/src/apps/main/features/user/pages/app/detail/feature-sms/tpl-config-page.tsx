"use client";

import {
  userSenderSmsTplConfigDel,
  UserSenderSmsTplConfigItemType,
  userSenderSmsTplConfigList,
} from "@shared/apis/user/sender-sms";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import {
  DEFAULT_PAGE_SIZE,
  PagePagination,
  useCountNumManager,
} from "@apps/main/lib/pagination-utils";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import { useIsMobile } from "@shared/hooks/use-mobile";
import {
  cn,
  formatServerError,
  formatTime,
  getQueryResponseData,
  TIME_STYLE,
} from "@shared/lib/utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-sms/tpl-config";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import { ExternalLink, Trash2 } from "lucide-react";
import React from "react";
import { featureSmsModuleConfig } from "../nav-info";
import { TplConfigDrawer } from "./tpl-config-drawer";
import {
  TplConfigFilterFormSchema
} from "./tpl-config-schema";

export default function AppDetailFeatureSmsTplConfigPage() {
  const { appId } = Route.useParams();
  const toast = useToast();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();
  const [drawerOpen, setDrawerOpen] = React.useState(false);

  // 从 URL 搜索参数获取过滤条件
  const filterParam = Route.useSearch();

  const currentPage = filterParam.page || 1;
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 过滤条件
  const filters = {
    tpl: filterParam.tpl || null,
  };

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  // 获取模板配置列表数据
  const {
    data: configData,
    isSuccess,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: [
      "tpl-config-list",
      appId,
      filterParam.tpl,
      currentPage,
      currentLimit,
    ],
    queryFn: async ({ signal }) => {
      const result = await userSenderSmsTplConfigList(
        {
          app_id: Number(appId),
          tpl: filterParam.tpl,
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
  isSuccess && countNumManager.handlePageQueryResult(configData);

  // 从查询结果中提取数据
  const configs = getQueryResponseData<UserSenderSmsTplConfigItemType[]>(configData, []);

  // 删除配置
  const deleteMutation = useMutation({
    mutationFn: (id: number) => userSenderSmsTplConfigDel({ config_id: id }),
    onSuccess: () => {
      toast.success("配置删除成功");
      queryClient.invalidateQueries({ queryKey: ["tpl-config-list", appId] });
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
    queryClient.refetchQueries({ queryKey: ["tpl-config-list"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["tpl-config-list"] });
  };

  // 定义表格列配置
  const columns: ColumnDef<UserSenderSmsTplConfigItemType>[] = [
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
      header: "配置名称",
      cell: ({ getValue }) => (
        <div className="font-medium">{getValue<string>()}</div>
      ),
    },
    {
      accessorKey: "tpl_key",
      header: "模板Key",
      cell: ({ getValue }) => (
        <div className="font-mono text-sm">{getValue<string>()}</div>
      ),
    },
    {
      accessorKey: "setting_name",
      header: "短信配置",
      cell: ({ row }) => (
        <div className="text-sm">
          <div className="font-medium">{row.original.setting_name}</div>
          <div className="text-xs text-muted-foreground">
            {row.original.setting_key}
          </div>
        </div>
      ),
    },
    {
      accessorKey: "config_data",
      header: "模板配置",
      cell: ({ row }) => {
        const configData = row.original.config_data;
        return (
          <div className="text-xs font-mono space-y-1">
            {configData?.aliyun_sms_tpl && (
              <div className="text-muted-foreground">
                阿里云模板: {configData.aliyun_sms_tpl}
              </div>
            )}
            {configData?.aliyun_sign_name && (
              <div className="text-muted-foreground">
                签名: {configData.aliyun_sign_name}
              </div>
            )}
            {configData?.sign_name && (
              <div className="text-muted-foreground">
                签名: {configData.sign_name}
              </div>
            )}
            {configData?.template_id && (
              <div className="text-muted-foreground">
                模板ID: {configData.template_id}
              </div>
            )}
            {configData?.signature && (
              <div className="text-muted-foreground">
                签名: {configData.signature}
              </div>
            )}
            {configData?.sender && (
              <div className="text-muted-foreground">
                发送者: {configData.sender}
              </div>
            )}
          </div>
        );
      },
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
      header: () => <div className="text-center">操作</div>,
      size: 120,
      cell: ({ row }) => {
        const config = row.original;

        return (
          <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <ConfirmDialog
                title="确认删除"
                description={
                  <div>
                    确定要删除配置 <strong>{config.name}</strong> 吗？
                    <br />
                    删除后无法恢复。
                  </div>
                }
                onConfirm={() => handleDelete(Number(config.id))}
              >
                <Button
                  size="sm"
                  variant="ghost"
                  className={cn("h-7 px-2 ")}
                  title="删除"
                >
                  <Trash2 className="h-4 w-4" />
                  <span className="ml-2">删除</span>
                </Button>
              </ConfirmDialog>
            </DataTableActionItem>
          </DataTableAction>
        );
      },
    },
  ];

  return (
    <>
      <AppDetailNavContainer
        {...featureSmsModuleConfig}
        actions={
          <Button
            variant="outline"
            size="sm"
            onClick={() => setDrawerOpen(true)}
          >
            <ExternalLink className=" h-4 w-4" />
           <span className="ml-2">新增配置</span>
          </Button>
        }
      >
        <div className="flex flex-col min-h-0 space-y-3">
          <div className="flex-shrink-0 mb-1 sm:mb-4">
            {/* 过滤器 */}
            <FilterContainer
              defaultValues={{
                tpl: filterParam.tpl,
              }}
              resolver={zodResolver(TplConfigFilterFormSchema) as any}
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
                  {/* 模板过滤 */}
                  <FilterInput
                    name="tpl"
                    label="模板Key"
                    placeholder="如: verify_code"
                    type="text"
                    layoutParams={layoutParams}
                  />

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
      </AppDetailNavContainer>

      <TplConfigDrawer
        appId={appId}
        open={drawerOpen}
        onOpenChange={setDrawerOpen}
      />
    </>
  );
}

// 导出 schema 供路由使用
export { TplConfigFilterParamSchema } from './tpl-config-schema';

