import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import {
  DEFAULT_PAGE_SIZE,
  OffsetPagination,
  PAGE_SIZE_OPTIONS,
  useCountNumManager,
  useOffsetPaginationHandlers,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-sms/list";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  userSenderSmsMessageCancel,
  UserSenderSmsMessageItemType,
  userSenderSmsMessageList
} from "@shared/apis/user/sender-sms";
import { DataTable } from "@shared/components/custom//table";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import { MaskedText } from "@shared/components/custom/text/masked-text";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import {
  cn,
  extractMinMax,
  formatServerError,
  formatTime,
  getQueryResponseData,
  getQueryResponseNext,
  TIME_STYLE
} from "@shared/lib/utils";
import { createCopyWithToast } from "@shared/lib/utils/copy-utils";
import { type LimitDataType } from "@shared/types/base-schema";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import { Copy, Eye, FileText, Settings, X } from "lucide-react";
import { useState } from "react";
import { featureSmsModuleConfig } from "../nav-info";
import { ListContentDrawer } from "./list-content-drawer";
import { ListLogsDrawer } from "./list-logs-drawer";
import { ListNotifyDrawer } from "./list-notify-drawer";
import {
  SmsListFilterFormSchema
} from "./list-schema";

export default function AppDetailFeatureSmsListPage() {
  // user\app_sender_smser\mapping.md
  // user\app_sender_smser\message_cancel.md
  // user\app_sender_smser\message_list.md
  // user\app_sender_smser\message_logs.md
  // user\app_sender_smser\message_view.md

  const { appId } = Route.useParams()
  const [notifyDrawerOpen, setNotifyDrawerOpen] = useState(false)

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
      <AppDetailNavContainer {...featureSmsModuleConfig}
        actions={
          <Button variant="outline" size="sm" onClick={() => setNotifyDrawerOpen(true)}>
            <Settings className={cn("mr-2 h-4 w-4")} />
            通知配置
          </Button>
        }>
        <AppDetailFeatureSmsListContent dictData={dictData} />
      </AppDetailNavContainer>
      <ListNotifyDrawer
        appId={String(appId)}
        open={notifyDrawerOpen}
        onOpenChange={setNotifyDrawerOpen}
      />
    </>
  )
}
interface AppDetailFeatureSmsListContentProps {

  dictData: TypedDictData<["user_sender_sms"]>;
}

function AppDetailFeatureSmsListContent({ dictData }: AppDetailFeatureSmsListContentProps) {
  const { appId } = Route.useParams();
  const queryClient = useQueryClient();
  const { success: showSuccess, error: showError } = useToast();
  const navigate = useNavigate();


  // 创建复制函数
  const copyText = createCopyWithToast(showSuccess, showError);

  // 从 URL 搜索参数获取过滤条件
  const filterParam = Route.useSearch();
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 日志抽屉状态
  const [logsDrawerOpen, setLogsDrawerOpen] = useState(false);
  const [selectedMessage, setSelectedMessage] = useState<UserSenderSmsMessageItemType | null>(null);
  // 详细信息抽屉状态
  const [detailDrawerOpen, setDetailDrawerOpen] = useState(false);
  const [detailMessage, setDetailMessage] = useState<UserSenderSmsMessageItemType | null>(null);

  // 过滤条件从 URL 参数获取
  const filters = {
    status: filterParam.status || null,
    tpl_key: filterParam.tpl_key || null,
    mobile: filterParam.mobile || null,
    snid: filterParam.snid || null,
  };

  // 分页状态 - 直接从 URL 参数派生，无需 useState
  const pagination: LimitDataType = {
    pos: filterParam.pos || null,
    limit: currentLimit,
    forward: filterParam.forward || false,
    more: true,
    eq_pos: filterParam.eq_pos || false,
  };

  // 搜索导航函数
  const searchGo = useSearchNavigate(navigate, filterParam);

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  // 获取消息列表数据
  const { data: messageData, isSuccess, isLoading, isError, error } = useQuery({
    queryKey: [
      "userSenderSmsMessageList",
      appId,
      pagination.pos,
      currentLimit,
      pagination.forward,
      pagination.more,
      pagination.eq_pos,
      filters.status,
      filters.tpl_key,
      filters.mobile,
      filters.snid,
    ],
    queryFn: ({ signal }) =>
      userSenderSmsMessageList(
        {
          app_id: Number(appId),
          limit: {
            eq_pos: pagination.eq_pos,
            pos: pagination.pos,
            limit: currentLimit,
            forward: pagination.forward,
            more: pagination.more,
          },
          count_num: countNumManager.getCountNum(),
          status: filters.status,
          tpl_key: filters.tpl_key || undefined,
          to_mobile: filters.mobile || undefined,
          snid: filters.snid ? Number(filters.snid) : undefined,
        },
        { signal }
      ),
    placeholderData: (previousData) => previousData,
  });

  // 处理 Limit 分页查询结果（自动提取 total 和 next）
  isSuccess && countNumManager.handleLimitQueryResult(messageData);

  // 获取消息列表数据
  const messages = getQueryResponseData<UserSenderSmsMessageItemType[]>(messageData, []);
  const nextPageStartPos = getQueryResponseNext(messageData);

  // 取消发送消息的mutation
  const cancelMessageMutation = useMutation({
    mutationFn: (params: { message_id: number }) =>
      userSenderSmsMessageCancel(params),
    onSuccess: () => {
      showSuccess("短信发送已取消");
      // 刷新列表数据
      queryClient.invalidateQueries({
        queryKey: ["userSenderSmsMessageList"],
      });
    },
    onError: (error: any) => {
      showError(formatServerError(error));
    },
  });

  // 取消发送
  const handleCancelMessage = async (messageId: number) => {
    await cancelMessageMutation.mutateAsync({ message_id: messageId });
  };

  // 打开日志抽屉
  const handleOpenLogs = (message: UserSenderSmsMessageItemType) => {
    setSelectedMessage(message);
    setLogsDrawerOpen(true);
  };

  // 使用统一的分页处理器
  const { handleNextPage, handlePrevPage, canGoNext, canGoPrev } =
    useOffsetPaginationHandlers({
      ...extractMinMax(messages, 'id', 'minId', 'maxId'),
      pagination,
      nextPageStartPos,
      searchGo,
      defaultForward: false, // 从大到小排序（新到旧）
    });

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["userSenderSmsMessageList"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["userSenderSmsMessageList"] });
  };

  // 字典数据已加载，创建状态映射器
  const smsStatus = createStatusMapper(
    {
      1: "info",      // 待发送
      2: "success",   // 已发送
      3: "danger",    // 发送失败
      4: "warning",   // 已取消
      5: "success",   // 已接收
    },
    (status) =>
      dictData.sms_send_status?.getLabel(String(status)) || String(status),
  );

  // 定义表格列配置
  const columns: ColumnDef<UserSenderSmsMessageItemType>[] = [
    {
      accessorKey: "snid",
      header: ({ column }) => (
        <div className="sm:text-right min-w-[50px] py-1">ID</div>
      ),
      size: 60, // 进一步减小第一列宽度
      cell: ({ getValue }) => (
        <div className="font-mono text-xs sm:text-right min-w-[50px] py-1">
          {getValue<number>()}
        </div>
      ),
    },
    {
      accessorKey: "mobile",
      header: "手机号",
      cell: ({ getValue }) => {
        const mobile = getValue<string>();
        return (
          <div className="max-w-[150px] py-1">
            <MaskedText
              text={mobile}
              type="phone"
              clickable={true}
              className="text-sm"
              onRevealedClick={() => copyText(mobile, "手机号已复制")}
            />
          </div>
        );
      },
    },
    {
      accessorKey: "tpl_key",
      header: "模板",
      cell: ({ getValue }) => {
        const tplKey = getValue<string>();
        return (
          <div className="py-1">
            <code
              className="rounded bg-muted px-[0.3rem] py-[0.1rem] text-xs cursor-pointer hover:bg-muted/80 transition-colors"
              onClick={() => copyText(tplKey, "模板Key已复制")}
              title="点击复制模板Key"
            >
              {tplKey}
              <Copy className="inline-block ml-1 h-3 w-3 opacity-50" />
            </code>
          </div>
        );
      },
    },
    {
      accessorKey: "status",
      header: "状态",
      cell: ({ getValue, row }) => {
        const status = getValue<number>();
        const on_task = row.original.on_task;
        const now_send = row.original.now_send;
        return (
          <div className="py-1">
            <Badge className={smsStatus.getClass(status)}>
              {smsStatus.getText(status)}
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
      accessorKey: "add_time",
      header: "添加时间",
      cell: ({ getValue }) => {
        const addTime = getValue<Date | null>();
        return (
          <div className="text-xs py-1">
            {addTime ? formatTime(addTime, TIME_STYLE.RELATIVE_ELEMENT) : "-"}
          </div>
        );
      },
    },
    {
      id: "actions",
      header: () => <div className="text-center py-1">操作</div>,
      cell: ({ row }) => {
        const message = row.original;


        return (
          <DataTableAction className="justify-end sm:justify-center gap-1">
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <Button
                variant="ghost"
                size="sm"
                className={cn("h-auto px-2 py-1")}
                title="查看内容"
                onClick={() => {
                  setDetailMessage(message);
                  setDetailDrawerOpen(true);
                }}
              >
                <FileText className="h-3 w-3" />
                <span className="text-xs ml-1">详细信息</span>
              </Button>
            </DataTableActionItem>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <Button
                variant="ghost"
                size="sm"
                className={cn("h-auto px-2 py-1")}
                title="查看日志"
                onClick={() => handleOpenLogs(message)}
              >
                <Eye className="h-3 w-3" />
                <span className="text-xs  ml-1">查看日志</span>
              </Button>
            </DataTableActionItem>
            {message.status === 1 && (
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                {/* 只有状态为发送中(1)时才显示取消按钮 */}

                <ConfirmDialog
                  title="确认取消发送"
                  description={
                    <>
                      您确定要取消发送这条短信吗？取消后将无法恢复。
                      <br />
                      手机号：
                      <MaskedText
                        text={message.mobile}
                        type="phone"
                        clickable={true}
                        className="inline"
                      />
                    </>
                  }
                  onConfirm={async () =>
                    await handleCancelMessage(message.id)
                  }
                >
                  <Button
                    variant="ghost"
                    size="sm"
                    className={cn("h-auto px-2 py-1")}
                    title="取消发送"
                  >
                    <X className="h-3 w-3 mr-1" />
                    <span className="text-xs">取消发送</span>
                  </Button>
                </ConfirmDialog>
              </DataTableActionItem>
            )}
          </DataTableAction>
        );
      },
    },
  ];

  return (
    <>

      <div className="flex flex-col min-h-0 space-y-6">
        <div className="flex-shrink-0 mb-1 sm:mb-4">
          {/* 过滤器 */}
          <FilterContainer
            defaultValues={{
              status: filterParam.status?.toString(),
              tpl_key: filterParam.tpl_key,
              mobile: filterParam.mobile,
              snid: filterParam.snid,
            }}
            resolver={zodResolver(SmsListFilterFormSchema) as any}
            onSubmit={(data) => {
              // zod schema 已经处理了类型转换和空值清理
              const transformedData = data as {
                status?: number;
                tpl_key?: string;
                mobile?: string;
                snid?: string;
              };
              searchGo({
                status: transformedData.status,
                tpl_key: transformedData.tpl_key,
                mobile: transformedData.mobile,
                snid: transformedData.snid,
                pos: null, // 重置分页位置
                forward: false,
                eq_pos: false,
              });
            }}
            onReset={() => {
              searchGo({
                pos: null,
                limit: currentLimit,
                forward: false,
                eq_pos: false,
                status: undefined,
                tpl_key: undefined,
                mobile: undefined,
                snid: undefined,
              });
            }}
            countComponent={
              <FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />
            }
            className="bg-card rounded-lg border shadow-sm relative"
          >
            {(layoutParams, form) => (
              <div className="flex-1 flex flex-wrap items-end gap-3">
                {/* 状态过滤 */}
                {dictData.sms_send_status && (
                  <FilterDictSelect
                    name="status"
                    placeholder="选择状态"
                    label="状态"
                    disabled={isLoading}
                    dictData={dictData.sms_send_status}
                    layoutParams={layoutParams}
                    allLabel="全部"
                  />
                )}

                {/* ID过滤 */}
                <FilterInput
                  name="snid"
                  placeholder="输入ID"
                  label="ID"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                />

                {/* 手机号过滤 */}
                <FilterInput
                  name="mobile"
                  placeholder="输入手机号"
                  label="手机号"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                />

                {/* 模板Key过滤 */}
                <FilterInput
                  name="tpl_key"
                  placeholder="输入模板Key"
                  label="模板Key"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                />

                {/* 动作按钮区域 */}
                <div className={cn(layoutParams.isMobile ? "w-full" : "flex-shrink-0")}>
                  <FilterActions
                    form={form}
                    loading={isLoading}
                    layoutParams={layoutParams}
                    onRefreshSearch={clearCacheAndReload}
                  />
                </div>
              </div>
            )}
          </FilterContainer>
        </div>
        {/* 加 min-h-0 防止内容撑开导致出现额外空白或双滚动条 */}
        <div className="flex-1 flex flex-col overflow-hidden min-h-0">
          {/* 数据表格 */}
          <DataTable
            data={messages}
            columns={columns}
            loading={isLoading}
            error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}

            scrollSnapDelay={300}
            leftStickyColumns={[{ column: 0, minWidth: "160px", maxWidth: "160px" }]}
            className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b"
          />

          {/* 分页控件 */}
          <div className="flex-shrink-0 pt-4">
            {(countNumManager.getTotal() ?? 0) > 0 && (
              <OffsetPagination
                limit={currentLimit}
                hasNext={canGoNext}
                canGoPrev={canGoPrev}
                total={countNumManager.getTotal()}
                currentPageSize={messages.length}
                loading={isLoading}
                onPrevious={handlePrevPage}
                onNext={handleNextPage}
                onRefresh={refreshData}
                showRefresh={true}
                showPageSize={true}
                pageSizeOptions={PAGE_SIZE_OPTIONS}
                onPageSizeChange={(pageSize) => {
                  // 更新页面大小并重置到第一页
                  searchGo({
                    limit: pageSize,
                    pos: null, // 重置分页位置
                    forward: false,
                    eq_pos: false,
                  });
                }}
              />
            )}
          </div>
        </div>

        {selectedMessage && (
          <ListLogsDrawer
            message={selectedMessage}
            isOpen={logsDrawerOpen}
            onOpenChange={setLogsDrawerOpen}
            dictData={dictData}
          />
        )}
        {detailMessage && (
          <ListContentDrawer
            open={detailDrawerOpen}
            onOpenChange={setDetailDrawerOpen}
            message={detailMessage}
            statusClass={smsStatus.getClass(detailMessage.status)}
            statusText={smsStatus.getText(detailMessage.status)}
            dictData={dictData}
          />
        )}
      </div>
    </>
  )
}
