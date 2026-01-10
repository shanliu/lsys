"use client";

import {
  userSenderMailerTplConfigDel,
  UserSenderMailerTplConfigItemType,
  userSenderMailerTplConfigList,
} from "@shared/apis/user/sender-mailer";
import { SenderTplConfigView } from "@apps/main/components/local/sender-config/tpl-config-view";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import {
  DEFAULT_PAGE_SIZE,
  PagePagination,
  useCountNumManager,
} from "@apps/main/lib/pagination-utils";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import { useIsMobile } from "@shared/hooks/use-mobile";
import {
  cn,
  formatServerError,
  formatTime,
  getQueryResponseData,
  TIME_STYLE,
} from "@shared/lib/utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-mail/tpl-config";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import { FileText, Plus, Trash2 } from "lucide-react";
import React from "react";
import { featureMailModuleConfig } from "../nav-info";
import { TplConfigDetailDrawer } from "./tpl-config-detail-drawer";
import { TplConfigDrawer } from "./tpl-config-drawer";
import {
  TplConfigFilterFormSchema
} from "./tpl-config-schema";

export default function AppDetailFeatureMailTplConfigPage() {
  // user\app_sender_mailer\tpl_config_del.md
  // user\app_sender_mailer\tpl_config_list.md
  // user\app_sender_mailer\smtp_config_list.md
  // user\app_sender_mailer\smtp_config_add.md

  const { appId } = Route.useParams();
  const [tplConfigDrawerOpen, setTplConfigDrawerOpen] = React.useState(false);

  // 字典数据获取 - 统一在最顶层获取一次
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["user_sender_mailer"] as const);

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
    return <PageSkeletonTable variant="page"  />;
  }

  // 字典加载成功，渲染内容组件
  return (
    <>
      <AppDetailNavContainer
        {...featureMailModuleConfig}
        actions={
          <Button size="sm" variant="outline" onClick={() => setTplConfigDrawerOpen(true)}>
            <Plus className={cn("mr-2 h-4 w-4")} />
            新增配置
          </Button>
        }
      >
        <AppDetailFeatureMailTplConfigContent dictData={dictData} />
      </AppDetailNavContainer>

      <TplConfigDrawer
        appId={appId}
        open={tplConfigDrawerOpen}
        onOpenChange={setTplConfigDrawerOpen}
      />
    </>
  );
}

// 内容组件：负责内容加载和渲染
interface AppDetailFeatureMailTplConfigContentProps {
  dictData: TypedDictData<["user_sender_mailer"]>;
}

function AppDetailFeatureMailTplConfigContent({ dictData }: AppDetailFeatureMailTplConfigContentProps) {
  const { appId } = Route.useParams();
  const toast = useToast();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();
  const [detailDrawerOpen, setDetailDrawerOpen] = React.useState(false);
  const [detailConfig, setDetailConfig] = React.useState<UserSenderMailerTplConfigItemType | null>(null);

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
      const result = await userSenderMailerTplConfigList(
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
  const configs = getQueryResponseData<UserSenderMailerTplConfigItemType[]>(configData, []);

  // 删除配置
  const deleteMutation = useMutation({
    mutationFn: (id: number) => userSenderMailerTplConfigDel({ tpl_config_id: id }),
    onSuccess: () => {
      toast.success("配置删除成功");
      countNumManager.reset();
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
  const columns: ColumnDef<UserSenderMailerTplConfigItemType>[] = [
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
      header: "模板",
      cell: ({ getValue }) => (
        <div className="font-mono text-sm">{getValue<string>()}</div>
      ),
    },
    {
      accessorKey: "setting_key",
      header: "配置类型",
      cell: ({ getValue }) => {
        const type = getValue<string>();
        return (
          <div className="text-sm">
            {dictData.config_type.getLabel(String(type))}
          </div>
        );
      },
    },
    {
      accessorKey: "setting_name",
      header: "配置名称",
      cell: ({ row }) => (
        <div className="text-sm">{row.original.setting_name}</div>
      ),
    },
    {
      accessorKey: "config_data",
      header: "配置详情",
      cell: ({ row }) => (
        <SenderTplConfigView
          config_data={row.original.config_data}
          setting_key={row.original.setting_key}
          variant="simple"
        />
      ),
    },
    {
      accessorKey: "change_time",
      header: "修改时间",
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
        const config = row.original;

        return (
          <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <Button
                variant="ghost"
                size="sm"
                className={cn("h-auto px-2 py-1")}
                title="查看内容"
                onClick={() => {
                  setDetailConfig(config);
                  setDetailDrawerOpen(true);
                }}
              >
              <FileText className="h-3 w-3" />
              <span className="text-xs ml-1">详细信息</span>
              </Button>
            </DataTableActionItem>
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
                placeholder="如: welcome_email"
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

      {detailConfig && (
        <TplConfigDetailDrawer
          open={detailDrawerOpen}
          onOpenChange={setDetailDrawerOpen}
          config={detailConfig}
          dictData={dictData}
        />
      )}
    </div>
  );
}

// 导出 schema 供路由使用
export { TplConfigFilterParamSchema } from './tpl-config-schema';

