"use client";

import {
  userSenderMailerTplBodyDel,
  userSenderMailerTplBodyList,
  type UserSenderMailerTplBodyItemType
} from "@shared/apis/user/sender-mailer";
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

import { Route } from "@apps/main/routes/_main/user/app/$appId/features-mail/tpl";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import { Edit2, Plus, Trash2 } from "lucide-react";
import React from "react";
import { featureMailModuleConfig } from "../nav-info";
import { MailTplDrawer } from "./tpl-drawer";
import {
  MailTplFilterFormSchema
} from "./tpl-schema";

export default function AppDetailFeatureMailTplPage() {
  // user\app_sender_mailer\tpl_body_add.md
  // user\app_sender_mailer\tpl_body_del.md
  // user\app_sender_mailer\tpl_body_edit.md
  // user\app_sender_mailer\tpl_body_list.md

  const { appId } = Route.useParams();
  const toast = useToast();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();
  const [tplDrawerOpen, setTplDrawerOpen] = React.useState(false);
  const [editingTpl, setEditingTpl] = React.useState<UserSenderMailerTplBodyItemType | undefined>();

  // 从 URL 搜索参数获取过滤条件
  const filterParam = Route.useSearch();

  const currentPage = filterParam.page || 1;
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 过滤条件
  const filters = {
    tpl_id: filterParam.tpl_id || null,
  };

  // count_num 优化管理器
  const countNumManager = useCountNumManager(filters);

  // 获取模板列表数据
  const {
    data: tplData,
    isSuccess,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: [
      "mail-tpl-list",
      appId,
      filterParam.tpl_id,
      currentPage,
      currentLimit,
    ],
    queryFn: async ({ signal }) => {
      const result = await userSenderMailerTplBodyList(
        {
          app_id: appId,
          tpl_id: filterParam.tpl_id,
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

  // 处理 Page 分页查询结果
  isSuccess && countNumManager.handlePageQueryResult(tplData);

  // 从查询结果中提取数据
  const templates = getQueryResponseData<UserSenderMailerTplBodyItemType[]>(tplData, []);

  // 删除模板
  const deleteMutation = useMutation({
    mutationFn: (id: number) => userSenderMailerTplBodyDel({ id }),
    onSuccess: () => {
      toast.success("模板删除成功");
      queryClient.invalidateQueries({ queryKey: ["mail-tpl-list", appId] });
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
    queryClient.refetchQueries({ queryKey: ["mail-tpl-list"] });
  };

  // 清除缓存并重新加载数据
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["mail-tpl-list"] });
  };

  // 定义表格列配置
  const columns: ColumnDef<UserSenderMailerTplBodyItemType>[] = [
    {
      accessorKey: "id",
      header: () => <div className={cn(isMobile ? "" : "text-right")}>ID</div>,
      size: 80,
      cell: ({ getValue }) => (
        <div className={cn("font-mono text-sm", isMobile ? "" : "text-right")}>{getValue<number>()}</div>
      ),
    },
    {
      accessorKey: "tpl_id",
      header: "模板ID",
      cell: ({ getValue }) => (
        <div className="font-mono text-sm">{getValue<string>()}</div>
      ),
    },
    {
      accessorKey: "tpl_data",
      header: "模板内容",
      cell: ({ getValue }) => {
        const content = getValue<string>();
        return (
          <div className="max-w-md truncate text-sm text-muted-foreground">
            {content}
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
        const tpl = row.original;

        return (
          <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <Button
                variant="ghost"
                size="sm"
                className={cn("h-7 px-2")}
                onClick={() => {
                  setEditingTpl(tpl);
                  setTplDrawerOpen(true);
                }}
              >
                <Edit2 className="h-4 w-4" />
                <span className="ml-2">编辑</span>
              </Button>
            </DataTableActionItem>

            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <ConfirmDialog
                title="确认删除"
                description={
                  <div>
                    确定要删除模板 <strong>ID: {tpl.id}</strong> 吗？
                    <br />
                    删除后无法恢复。
                  </div>
                }
                onConfirm={() => handleDelete(Number(tpl.id))}
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
        {...featureMailModuleConfig}
        actions={
          <Button
            size="sm"
            variant="outline"
            onClick={() => {
              setEditingTpl(undefined);
              setTplDrawerOpen(true);
            }}
          >
            <Plus className={cn("mr-2 h-4 w-4")} />
            新增模板
          </Button>
        }
      >
        <div className="flex flex-col min-h-0 space-y-3">
          <div className="flex-shrink-0 mb-1 sm:mb-4">
            {/* 过滤器 */}
            <FilterContainer
              defaultValues={{
                tpl_id: filterParam.tpl_id,
              }}
              resolver={zodResolver(MailTplFilterFormSchema) as any}
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
                  {/* 模板ID过滤 */}
                  <FilterInput
                    name="tpl_id"
                    label="模板ID"
                    placeholder="输入模板ID"
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

          {/* 表格和分页容器 */}
          <div className="flex-1 flex flex-col min-h-0">
            {/* 数据表格 */}
            <div className="flex-1 overflow-hidden">
              <DataTable
                data={templates}
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

            {/* 分页控件 */}
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

      <MailTplDrawer
        tpl={editingTpl}
        open={tplDrawerOpen}
        onOpenChange={(open) => {
          setTplDrawerOpen(open);
          if (!open) {
            setEditingTpl(undefined);
          }
        }}
      />
    </>
  );
}

// 导出 schema 供路由使用
export { MailTplFilterParamSchema } from './tpl-schema';
