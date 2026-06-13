// docs\api\system\sender_mailer\mapping.md
// docs\api\system\sender_mailer\message_cancel.md
// docs\api\system\sender_mailer\message_list.md
// docs\api\system\sender_mailer\message_logs.md
// docs\api\system\sender_mailer\message_view.md
import { FilterBar } from "@apps/main/components/filter-bar/container";
import { FilterActions } from "@apps/main/components/filter-bar/filter-actions/filter-actions";
import { FilterResetButton } from "@apps/main/components/filter-bar/filter-actions/filter-reset-button";
import { FilterSearchButton } from "@apps/main/components/filter-bar/filter-actions/filter-search-button";
import { useAdminExportAction } from "@apps/main/hooks/use-admin-export-action";
import { EXPORT_TYPE_SYSTEM_MAILER_MESSAGE_LIST } from "@shared/apis/admin/export";
import { FilterDictSelect, FilterInput, FilterTotalCount } from "@apps/main/components/filter-bar/filter-fields";
import { ExportButton, ExportMobileButton, ExportSplitButton } from "@apps/main/components/export-manager/export-buttons";
import { ExportDrawer } from "@apps/main/components/export-manager/export-drawer";
import { useFilterBarForm } from "@apps/main/hooks/use-filter-bar-form";
import * as z from "zod";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import {
  CursorPagination,
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  useLimitCountNum,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/admin/email/send-log";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  systemSenderMailerMessageCancel,
  systemSenderMailerMessageList,
  type SystemSenderMailerMessageItemType,
} from "@shared/apis/admin/sender-mailer";
import { DataTable } from "@shared/components/custom//table";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import {
  cn,
  formatServerError,
  formatTime,
  getQueryResponseCursor,
  getQueryResponseData,
  TIME_STYLE,
} from "@shared/lib/utils";
import { formatTotalCount } from "@shared/lib/utils/format-utils";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ColumnDef } from "@tanstack/react-table";
import { Eye, FileText, X } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";
import { SendLogDetailDrawer } from "./send-log-detail-drawer";
import { SendLogLogsDrawer } from "./send-log-logs-drawer";
import {
  EmailLogFilterFormSchema
} from "./send-log-schema";

export function SendLogPage() {
  // 字典数据获取 - 统一在最顶层获取一次
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["admin_sender_mailer", "admin_export"] as const);

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
  return <SendLogContent dictData={dictData} />;
}

// 内容组件：负责内容加载和渲染
interface SendLogContentProps {
  dictData: TypedDictData<["admin_sender_mailer", "admin_export"]>;
}

function SendLogContent({ dictData }: SendLogContentProps) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  // 获取 URL search 参数
  const filterParam = Route.useSearch();
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 详情抽屉状态
  const [detailDrawer, setDetailDrawer] = useState({
    open: false,
    message: null as SystemSenderMailerMessageItemType | null,
  });

  // 日志抽屉状态
  const [logsDrawer, setLogsDrawer] = useState({
    open: false,
    message: null as SystemSenderMailerMessageItemType | null,
  });

  // 取消操作状态
  const [cancelingIds, setCancelingIds] = useState<Set<number>>(new Set());

  // 过滤条件从 URL 参数获取
  const filters = {
    tpl_key: filterParam.tpl_key || null,
    status: filterParam.status || null,
    body_id: filterParam.body_id || null,
    snid: filterParam.snid || null,
    to_mail: filterParam.to_mail || null,
  };

  // 分页状态 - 直接从 URL 参数派生，无需 useState
  const pagination = {
    pos: filterParam.pos || null,
    limit: currentLimit,
    forward: filterParam.forward ?? true,
    more: true,
  };

  // 搜索导航函数
  const searchGo = useSearchNavigate(navigate, filterParam);

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useLimitCountNum(filters);

  // 获取邮件日志数据
  const { data: messageData, isSuccess: messageIsSuccess, isLoading: messageIsLoading, isError, error } = useQuery({
    queryKey: [
      "systemSenderMailerMessageList",
      pagination.pos,
      pagination.limit,
      pagination.forward,
      filters.tpl_key,
      filters.status,
      filters.body_id,
      filters.snid,
      filters.to_mail,
    ],
    queryFn: ({ signal }) =>
      systemSenderMailerMessageList(
        {
          limit: {
            pos: pagination.pos,
            limit: pagination.limit,
            forward: pagination.forward,
            more: pagination.more,
          },
          count_num: countNumManager.getCountNum(),
          ...(filters.tpl_key && { tpl_key: filters.tpl_key }),
          ...(filters.status !== undefined &&
            filters.status !== null && { status: filters.status }),
          ...(filters.body_id !== undefined &&
            filters.body_id !== null && { body_id: filters.body_id }),
          ...(filters.snid && { snid: filters.snid }),
          ...(filters.to_mail && { to_mail: filters.to_mail }),
        },
        { signal },
      ),
    placeholderData: (previousData) => previousData,
  });

  // 处理 Limit 分页查询结果（自动提取 total 和 next）
  messageIsSuccess && countNumManager.handleQueryResult(messageData);

  // 取消邮件发送
  const cancelMutation = useMutation({
    mutationFn: (messageId: number) =>
      systemSenderMailerMessageCancel({ message_id: messageId }),
    onMutate: (messageId) => {
      setCancelingIds((prev) => new Set(prev).add(messageId));
    },
    onSuccess: (_, messageId) => {
      toast.success("邮件发送已取消");
      queryClient.invalidateQueries({
        queryKey: ["systemSenderMailerMessageList"],
      });
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    },
    onSettled: (_, __, messageId) => {
      setCancelingIds((prev) => {
        const newSet = new Set(prev);
        newSet.delete(messageId);
        return newSet;
      });
    },
  });

  // 获取API响应数据
  const messages = getQueryResponseData<SystemSenderMailerMessageItemType[]>(
    messageData,
    [],
  );
  const cursorData = getQueryResponseCursor(messageData);

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({
      queryKey: ["systemSenderMailerMessageList"],
    });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["systemSenderMailerMessageList"] });
  };

  // 导出操作 hook
  const exportAction = useAdminExportAction({
    exportType: EXPORT_TYPE_SYSTEM_MAILER_MESSAGE_LIST,
    params: {
      tpl_key: filters.tpl_key ?? undefined,
      status: filters.status ?? undefined,
      body_id: filters.body_id ?? undefined,
      snid: filters.snid ?? undefined,
      to_mail: filters.to_mail ?? undefined,
    },
  });

  const filterForm = useFilterBarForm<z.infer<typeof EmailLogFilterFormSchema>>({
    defaultValues: {
      tpl_key: filterParam.tpl_key,
      status: filterParam.status,
      body_id: filterParam.body_id,
      snid: filterParam.snid,
      to_mail: filterParam.to_mail,
    },
    resolver: zodResolver(EmailLogFilterFormSchema) as any,
    initValues: {
      tpl_key: undefined,
      status: undefined,
      body_id: undefined,
      snid: undefined,
      to_mail: undefined,
    },
    onSubmit: (data) => {
      navigate({ search: { ...data, pos: null, forward: true } as any });
    },
    onReset: () => {
      navigate({ search: { pos: null, limit: currentLimit, forward: true } as any });
    },
  });

  // 处理查看详情（用 useMemo 因为在 columns useMemo 中使用）
  const handleViewDetail = useMemo(
    () => (message: SystemSenderMailerMessageItemType) => {
      setDetailDrawer({ open: true, message });
    },
    [],
  );

  // 处理查看日志（用 useMemo 因为在 columns useMemo 中使用）
  const handleViewLogs = useMemo(
    () => (message: SystemSenderMailerMessageItemType) => {
      setLogsDrawer({ open: true, message });
    },
    [],
  );

  // 处理取消发送（用 useMemo 因为在 columns useMemo 中使用）
  const handleCancelMessage = useMemo(
    () => (messageId: number) => {
      cancelMutation.mutate(messageId);
    },
    [cancelMutation],
  );

  // 关闭详情抽屉
  const handleCloseDetailDrawer = () => {
    setDetailDrawer({ open: false, message: null });
  };

  // 关闭日志抽屉
  const handleCloseLogsDrawer = () => {
    setLogsDrawer({ open: false, message: null });
  };

  const isLoading = messageIsLoading;

  // 创建状态映射器
  const emailStatus = createStatusMapper(
    {
      1: "info",      // 待发送
      2: "success",   // 已发送
      3: "danger",    // 发送失败
      4: "warning",   // 已取消
      5: "success",   // 已接收
    },
    (status) => dictData.mail_send_status?.getLabel(String(status)) || String(status),
  );

  // 定义表格列
  const columns = useMemo<ColumnDef<SystemSenderMailerMessageItemType>[]>(
    () => [
      {
        accessorKey: "snid",
        header: () => <div className="sm:text-right">ID</div>,
        size: 80,
        cell: ({ getValue }) => (
          <div className="font-mono text-xs sm:text-right">{getValue<string>()}</div>
        ),
      },
      {
        accessorKey: "to_mail",
        header: "收件邮箱",
        size: 160,
        cell: ({ getValue }) => (
          <CopyableText value={getValue<string>()} className="text-sm" />
        ),
      },
      {
        accessorKey: "tpl_key",
        header: "模板键值",
        size: 200,
        cell: ({ getValue }) => (
          <Badge variant="outline" className="max-w-[180px] truncate">{getValue<string>()}</Badge>
        ),
      },

      {
        accessorKey: "status",
        header: "状态",
        size: 100,
        cell: ({ getValue, row }) => {
          const status = getValue<number>();
          const on_task = row.original.on_task;
          const now_send = row.original.now_send;
          if (!emailStatus) return String(status);
          return (
            <div className="py-1">
              <Badge className={cn(emailStatus.getClass(status), "whitespace-nowrap")}>
                {emailStatus.getText(status)}
                {on_task ? "(发送中)" : ""}
                {now_send ? "(即将发送)" : ""}
              </Badge>
            </div>
          );
        },
      },
      {
        accessorKey: "send_time",
        header: "发送时间",
        size: 140,
        cell: ({ row }) => {
          const { send_time, expected_time } = row.original;

          // 如果已经有实际发送时间，优先显示实际发送时间
          if (send_time) {
            return (
              <div className="text-xs py-1">
                {formatTime(send_time, TIME_STYLE.ABSOLUTE_ELEMENT)}
              </div>
            );
          }

          // 否则显示预期发送时间
          if (expected_time) {
            return (
              <div className="text-xs py-1">
                {formatTime(expected_time, TIME_STYLE.ABSOLUTE_ELEMENT)}
              </div>
            );
          }

          return <div className="text-xs py-1">-</div>;
        },
      },
      {
        id: "actions",
        header: () => <div className="text-center">操作</div>,
        size: 80,
        cell: ({ row }) => {
          const message = row.original;
          const isCanceling = cancelingIds.has(message.id);
          // 状态 0 表示发送中
          const isSending = message.status === 0;

          return (
            <DataTableAction className="justify-end sm:justify-center">
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 px-2"
                  onClick={() => handleViewDetail(message)}
                  title="查看详情"
                >
                  <Eye className="h-4 w-4" />
                  <span className="ml-2">查看详情</span>
                </Button>
              </DataTableActionItem>
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 px-2"
                  onClick={() => handleViewLogs(message)}
                  title="发送日志"
                >
                  <FileText className="h-4 w-4" />
                  <span className="ml-2">发送日志</span>
                </Button>
              </DataTableActionItem>
              {isSending ? (
                <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 px-2 "
                    onClick={() => handleCancelMessage(message.id)}
                    disabled={isCanceling}
                    title="取消发送"
                  >
                    <X className="h-4 w-4" />
                    <span className="ml-2">{isCanceling ? "取消中..." : "取消发送"}</span>
                  </Button>
                </DataTableActionItem>
              ) : null}
            </DataTableAction>
          );
        },
      },
    ],
    [emailStatus, cancelingIds, handleCancelMessage, handleViewDetail, handleViewLogs],
  );

  return (
    <>
      <div className="container mx-auto p-4 max-w-[1600px] flex flex-col min-h-0 space-y-5">
        {/* 搜索和过滤 */}
        <div className="flex-shrink-0 mb-1 sm:mb-4">
          <FilterBar form={filterForm} className={cn("bg-card rounded-lg border shadow-sm relative")}>
            <FilterBar.Summary>
              <FilterTotalCount value={formatTotalCount(countNumManager.getTotalInfo())} loading={isLoading} />
            </FilterBar.Summary>
            <FilterBar.MobileExtra>
              <ExportMobileButton activeCount={exportAction.activeCount} isLoading={exportAction.activeCount > 0} onClick={exportAction.openDrawer} />
            </FilterBar.MobileExtra>
            {/* 序列号过滤 */}
            <div className="flex-1 min-w-[140px] max-w-[200px]">
              <FilterInput name="snid" placeholder="输入ID" label="ID" disabled={isLoading} />
            </div>
            {/* 模板键值过滤 */}
            <div className="flex-1 min-w-[160px] max-w-[250px]">
              <FilterInput name="tpl_key" placeholder="搜索模板键值" label="模板键值" disabled={isLoading} />
            </div>
            {/* 状态过滤 */}
            <div className="flex-1 min-w-[140px] max-w-[200px]">
              <FilterDictSelect name="status" placeholder="选择状态" label="发送状态" disabled={isLoading} dictData={dictData.mail_send_status} allLabel="全部" />
            </div>
            {/* 收件邮箱过滤 */}
            <div className="flex-1 min-w-[180px] max-w-[280px]">
              <FilterInput name="to_mail" placeholder="输入收件邮箱" label="收件邮箱" disabled={isLoading} />
            </div>
            {/* 动作按钮区域 */}
            <div className={cn("flex-shrink-0")}>
              <FilterActions>
                <FilterSearchButton loading={isLoading} onRefreshSearch={clearCacheAndReload} />
                <FilterResetButton loading={isLoading} />
                <FilterBar.DesktopOnly>
                  <ExportSplitButton activeCount={exportAction.activeCount} onSubmitExport={exportAction.submit}
                    onViewHistory={exportAction.openDrawer} isSubmitting={exportAction.isSubmitting} />
                </FilterBar.DesktopOnly>
              </FilterActions>
            </div>
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
            statusDict={dictData.export_task_status!}
            tasks={exportAction.tasks} totalCount={exportAction.totalCount}
            currentPage={exportAction.currentPage} totalPages={exportAction.totalPages}
            onPageChange={exportAction.setPage} isLoading={exportAction.isLoadingTasks}
          />
        </div>

        {/* 表格和分页容器 */}
        <div className="flex-1 flex flex-col overflow-hidden min-h-0">
          {/* 数据表格 */}
          <div className="flex-1 min-h-0">
            <div className="h-full">
              <DataTable
                data={messages}
                columns={columns}
                leftStickyColumns={[{ column: 0, minWidth: "180px", maxWidth: "200px" }]}
                loading={isLoading}
                error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}
                className={cn("h-full [&_.data-table-row]:h-12 [&_td]:py-2 [&_th]:py-2 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b [&_.data-table-wrapper]:overflow-auto [&_.data-table-wrapper]:h-full")}
              />
            </div>
          </div>

          {/* 分页控件 */}
          <div className="flex-shrink-0 pt-4">
            {countNumManager.hasTotalInfo() && (
              <CursorPagination
                limit={currentLimit}
                cursorData={cursorData}
                searchGo={searchGo}
                totalInfo={countNumManager.getTotalInfo()}
                currentPageSize={messages.length}
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
            )}
          </div>
        </div>

        {/* 详情抽屉 */}
        {detailDrawer.message && (
          <SendLogDetailDrawer
            message={detailDrawer.message}
            open={detailDrawer.open}
            onClose={handleCloseDetailDrawer}
            dictData={dictData}
            emailStatus={emailStatus}
          />
        )}

        {/* 日志抽屉 */}
        {logsDrawer.message && (
          <SendLogLogsDrawer
            message={logsDrawer.message}
            open={logsDrawer.open}
            onClose={handleCloseLogsDrawer}
            logTypeDict={dictData.log_type}
            logStatusDict={dictData.log_status}
          />
        )}
      </div>
    </>
  );
}
