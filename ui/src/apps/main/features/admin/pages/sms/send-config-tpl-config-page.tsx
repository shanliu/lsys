"use client";

import {
  systemSenderSmsTplConfigDelete,
  SystemSenderSmsTplConfigItemType,
  systemSenderSmsTplConfigList,
} from "@shared/apis/admin/sender-sms";
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
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { Route } from "@apps/main/routes/_main/admin/sms/send-config/channel";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import { SmsSendConfigNavContainer } from "@apps/main/features/admin/components/ui/sms-send-config-nav";
import { useIsMobile } from "@shared/hooks/use-mobile";
import {
  cn,
  formatServerError,
  formatTime,
  getQueryResponseData,
  TIME_STYLE,
} from "@shared/lib/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type ColumnDef } from "@tanstack/react-table";
import { Eye, Plus, Trash2 } from "lucide-react";
import React from "react";
import { smsSendConfigModuleConfig } from "../nav-info";
import { SmsSendConfigTplConfigDetailDrawer } from "./send-config-tpl-config-detail-drawer";
import { SmsSendConfigTplConfigDrawer } from "./send-config-tpl-config-drawer";
import {
  SmsSendConfigTplFilterFormSchema,
  SmsSendConfigTplFilterParamSchema,
  SmsSendConfigTplFilterParamType,
} from "./send-config-tpl-config-schema";

// infer item type from schema

export function SmsSendConfigTplConfigPage() {
  // 从 URL 获取 filter 状态
  const filterParam = Route.useSearch();
  const navigate = Route.useNavigate();
  const searchGo = useSearchNavigate(navigate, filterParam);
  const onNavigate = (params: SmsSendConfigTplFilterParamType) => searchGo(params);
  const toast = useToast();
  const queryClient = useQueryClient();
  const isMobile = useIsMobile();
  const [drawerOpen, setDrawerOpen] = React.useState(false);
  const [detailDrawerOpen, setDetailDrawerOpen] = React.useState(false);
  const [selectedConfig, setSelectedConfig] = React.useState<SystemSenderSmsTplConfigItemType | null>(null);

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
      "admin-tpl-config-list",
      filterParam.tpl,
      currentPage,
      currentLimit,
    ],
    queryFn: async ({ signal }) => {
      const result = await systemSenderSmsTplConfigList(
        {
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
  const configs = getQueryResponseData<SystemSenderSmsTplConfigItemType[]>(configData, []);

  // 删除配置
  const deleteMutation = useMutation({
    mutationFn: (id: number) => systemSenderSmsTplConfigDelete({ tpl_config_id: id }),
    onSuccess: () => {
      toast.success("配置删除成功");
      queryClient.invalidateQueries({ queryKey: ["admin-tpl-config-list"] });
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
    queryClient.refetchQueries({ queryKey: ["admin-tpl-config-list"] });
  };

  // 定义表格列配置
  const columns: ColumnDef<SystemSenderSmsTplConfigItemType>[] = [
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
     size: 380,
      cell: ({ getValue }) => (
        <div className={cn("font-mono text-sm", isMobile ? "break-all" : "whitespace-nowrap")}>{getValue<string>()}</div>
      ),
    },
    {
      accessorKey: "setting_name",
      header: "短信配置",
      cell: ({ row }) => (
        <div className="text-sm">
          <div className="font-medium">{row.original.setting_name}</div>
          <div className="text-xs text-muted-foreground">
            ID: {row.original.setting_id}
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
              <div className="text-muted-foreground break-all">
                阿里云模板: {configData.aliyun_sms_tpl}
              </div>
            )}
            {configData?.aliyun_sign_name && (
              <div className="text-muted-foreground break-all">
                签名: {configData.aliyun_sign_name}
              </div>
            )}
            {configData?.sign_name && (
              <div className="text-muted-foreground break-all">
                签名: {configData.sign_name}
              </div>
            )}
            {configData?.template_id && (
              <div className="text-muted-foreground break-all">
                模板ID: {configData.template_id}
              </div>
            )}
            {configData?.signature && (
              <div className="text-muted-foreground break-all">
                签名: {configData.signature}
              </div>
            )}
            {configData?.sender && (
              <div className="text-muted-foreground break-all">
                发送者: {configData.sender}
              </div>
            )}
            {configData?.template_map && (
              <div className="text-muted-foreground break-all">
                映射: {configData.template_map}
              </div>
            )}
          </div>
        );
      },
    },
    {
      accessorKey: "change_time",
      header: "更新时间",
      size: 100,
      cell: ({ getValue }) => (
        <div className="text-sm text-muted-foreground">
          {formatTime(getValue<number>(), TIME_STYLE.RELATIVE_TEXT)}
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
                size="sm"
                variant="ghost"
                className={cn("h-7 px-2")}
                title="查看详情"
                onClick={() => {
                  setSelectedConfig(config);
                  setDetailDrawerOpen(true);
                }}
              >
                <Eye className="h-4 w-4" />
                 <span className="ml-2">详情</span>
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

  // 渲染加载错误状态
  if (isError) {
    return (
      <SmsSendConfigNavContainer className={cn("m-4 md:m-6")} {...smsSendConfigModuleConfig}>
        <CenteredError variant="content" error={error} onReset={refreshData} />
      </SmsSendConfigNavContainer>
    );
  }

  // 渲染加载中状态
  if (isLoading) {
    return (
      <SmsSendConfigNavContainer className={cn("m-4 md:m-6")} {...smsSendConfigModuleConfig}>
        <PageSkeletonTable variant="content" />
      </SmsSendConfigNavContainer>
    );
  }

  return (
    <SmsSendConfigNavContainer
      className={cn("m-4 md:m-6")}
      {...smsSendConfigModuleConfig}
      actions={
        <Button size="sm" variant="outline" onClick={() => setDrawerOpen(true)}>
          <Plus className={cn("mr-2 h-4 w-4")} />
          新增配置
        </Button>
      }
    >
      <div className="space-y-6">

        <FilterContainer
          defaultValues={{
            tpl: filterParam.tpl,
          }}
          resolver={zodResolver(SmsSendConfigTplFilterFormSchema) as any}
          onSubmit={(data) => {
            onNavigate({
              ...filterParam,
              ...data,
              page: 1,
            });
          }}
          onReset={() => {
            onNavigate({
              ...filterParam,
              tpl: undefined,
              page: 1,
            });
          }}
          countComponent={
            <FilterTotalCount
              total={countNumManager.getTotal() ?? 0}
              loading={isLoading}
            />
          }
        >
          {(layoutParams, form) => (
            <>
              <FilterInput
                name="tpl"
                label="模板Key"
                placeholder="请输入模板Key"
                type="text"
                layoutParams={layoutParams}
              />
              <FilterActions
                form={form}
                loading={isLoading}
                layoutParams={layoutParams}
              />
            </>
          )}
        </FilterContainer>

        <DataTable
          columns={columns}
          data={configs}
          loading={isLoading}
          error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}
        />

        <PagePagination
          currentPage={currentPage}
          pageSize={currentLimit}
          total={countNumManager.getTotal() || 0}
          loading={isLoading}
          onChange={(page: number) => {
            onNavigate({ ...filterParam, page });
          }}
          onPageSizeChange={(limit: number) => {
            onNavigate({ ...filterParam, limit, page: 1 });
          }}
        />
      </div>

      <SmsSendConfigTplConfigDrawer
        open={drawerOpen}
        onOpenChange={setDrawerOpen}
      />

      <SmsSendConfigTplConfigDetailDrawer
        config={selectedConfig}
        open={detailDrawerOpen}
        onClose={() => {
          setDetailDrawerOpen(false);
          setSelectedConfig(null);
        }}
      />
    </SmsSendConfigNavContainer>
  );
}
