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
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-mail/list";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  userSenderMailerMessageCancel,
  userSenderMailerMessageList,
  type UserSenderMailerMessageItemType,
} from "@shared/apis/user/sender-mailer";
import { DataTable } from "@shared/components/custom//table";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { MaskedText } from "@shared/components/custom/text/masked-text";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import {
  cn,
  extractMinMax,
  formatTime,
  getQueryResponseData,
  getQueryResponseNext,
  TIME_STYLE,
} from "@shared/lib/utils";
import { createCopyWithToast } from "@shared/lib/utils/copy-utils";
import { type LimitDataType } from "@shared/types/base-schema";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import { Eye, FileText, X } from "lucide-react";
import { useState } from "react";
import { featureMailModuleConfig } from "../nav-info";
import { ListContentDrawer } from "./list-content-drawer";
import { ListLogsDrawer } from "./list-logs-drawer";
import {
  MailListFilterFormSchema
} from "./list-schema";

export default function AppDetailFeatureMailListPage() {
  // user\app_sender_mailer\message_view.md
  // user\app_sender_mailer\message_logs.md
  // user\app_sender_mailer\message_list.md
  // user\app_sender_mailer\message_cancel.md
  // user\app_sender_mailer\mapping.md

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
    return <PageSkeletonTable variant="page" />;
  }

  // 字典加载成功，渲染内容组件
  return <AppDetailNavContainer {...featureMailModuleConfig}>
    <AppDetailFeatureMailListContent dictData={dictData} />
  </AppDetailNavContainer>;
}

// 内容组件：负责内容加载和渲染
interface AppDetailFeatureMailListContentProps {
  dictData: TypedDictData<["user_sender_mailer"]>;
}

function AppDetailFeatureMailListContent({ dictData }: AppDetailFeatureMailListContentProps) {
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
  const [selectedMessage, setSelectedMessage] = useState<UserSenderMailerMessageItemType | null>(null);
  // 详细信息抽屉状态
  const [detailDrawerOpen, setDetailDrawerOpen] = useState(false);
  const [detailMessage, setDetailMessage] = useState<UserSenderMailerMessageItemType | null>(null);

  // 过滤条件从 URL 参数获取
  const filters = {
    status: filterParam.status || null,
    tpl_id: filterParam.tpl_id || null,
    to_mail: filterParam.to_mail || null,
    snid: filterParam.snid || null,
  };

  // 分页状态 - 直接从 URL 参数派生，无需 useState
  const pagination: LimitDataType = {
    pos: filterParam.pos || null,
    limit: filterParam.limit || DEFAULT_PAGE_SIZE,
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
      "userSenderMailerMessageList",
      appId,
      pagination.pos,
      currentLimit,
      pagination.forward,
      pagination.more,
      pagination.eq_pos,
      filters.status,
      filters.tpl_id,
      filters.to_mail,
      filters.snid,
    ],
    queryFn: ({ signal }) =>
      userSenderMailerMessageList(
        {
          app_id: Number(appId),
          limit: pagination,
          count_num: countNumManager.getCountNum(),
          status: filters.status,
          tpl_id: filters.tpl_id,
          to_mail: filters.to_mail,
          snid: filters.snid,
        },
        { signal }
      )
    ,
    placeholderData: (previousData) => previousData,
  });
  console.log("messageData:", isError);

  // 处理 Limit 分页查询结果（自动提取 total 和 next）
  isSuccess && countNumManager.handleLimitQueryResult(messageData);

  // 获取消息列表数据
  const messages = getQueryResponseData<UserSenderMailerMessageItemType[]>(messageData, []);
  const nextPageStartPos = getQueryResponseNext(messageData);

  // 调试信息


  // 取消发送消息的mutation
  const cancelMessageMutation = useMutation({
    mutationFn: (params: { message_id: number }) =>
      userSenderMailerMessageCancel(params),
    onSuccess: () => {
      showSuccess("邮件发送已取消");
      // 刷新列表数据
      queryClient.invalidateQueries({
        queryKey: ["userSenderMailerMessageList"],
      });
    },
    onError: (error: any) => {
      showError(error?.response?.data?.message || "取消邮件发送失败");
    },
  });

  // 取消发送
  const handleCancelMessage = async (messageId: number) => {
    await cancelMessageMutation.mutateAsync({ message_id: messageId });
  };

  // 打开日志抽屉
  const handleOpenLogs = (message: UserSenderMailerMessageItemType) => {
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
    queryClient.refetchQueries({ queryKey: ["userSenderMailerMessageList"] });
  };

  // 清除缓存并重新加载数据（双击搜索按钮时）
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["userSenderMailerMessageList"] });
  };

  // 字典数据已加载，创建状态映射器
  const mailStatus = createStatusMapper(
    {
      1: "info",      // 待发送
      2: "success",   // 已发送
      3: "danger",    // 发送失败
      4: "warning",   // 已取消
      5: "success",   // 已接收
    },
    (status) =>
      dictData.mail_send_status?.getLabel(String(status)) || String(status),
  );

  // 定义表格列配置
  const columns: ColumnDef<UserSenderMailerMessageItemType>[] = [
    {
      accessorKey: "snid",
      header: ({ column }) => (
        <div className="sm:text-right min-w-[50px] py-1">ID</div>
      ),
      size: 60, // 进一步减小第一列宽度
      cell: ({ getValue }) => (
        <div className="font-mono text-xs sm:text-right min-w-[50px] py-1">
          {getValue<string>()}
        </div>
      ),
    },
    {
      accessorKey: "to_mail",
      header: "收件人",
      cell: ({ getValue }) => {
        const email = getValue<string>();
        return (
          <div className="max-w-[200px] py-1">
            <MaskedText
              text={email}
              type="email"
              clickable={true}
              className="text-sm"
              onRevealedClick={() => copyText(email, "邮箱已复制")}
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
            <CopyableText
              value={tplKey}
              message="模板ID已复制"
              title="点击复制模板ID"
              className="bg-muted"
              showIcon={true}
            />
          </div>
        );
      },
    },
    {
      accessorKey: "status",
      header: "状态",
      size: 100,
      cell: ({ row, getValue }) => {
        const status = getValue<number>();
        const on_task = row.original.on_task;
        const now_send = row.original.now_send;
        return (
          <div className="py-1">
            <Badge className={cn(mailStatus.getClass(status))}>
              {mailStatus.getText(status)}
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
      accessorKey: "add_time",
      header: "添加时间",
      size: 100,
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
      size: 80,
      cell: ({ row }) => {
        const message = row.original;
        return <DataTableAction className="justify-end sm:justify-center">
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
              <span className="text-xs ml-1">查看日志</span>
            </Button>
          </DataTableActionItem>
          {message.status === 1 ? (
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">

              <ConfirmDialog
                title="确认取消发送"
                description={
                  <>
                    您确定要取消发送这封邮件吗？取消后将无法恢复。
                    <br />
                    收件人：
                    <MaskedText
                      text={message.to_mail}
                      type="email"
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
                  <X className="h-3 w-3" />
                  <span className="text-xs ml-1">取消发送</span>
                </Button>
              </ConfirmDialog>

            </DataTableActionItem>) : null}
        </DataTableAction>
      },
    },
  ];

  return (
    <>

      <div className="flex flex-col min-h-0 space-y-3">
        <div className="flex-shrink-0 mb-1 sm:mb-4">
          {/* 过滤器 */}
          <FilterContainer
            defaultValues={{
              status: filterParam.status?.toString(),
              tpl_id: filterParam.tpl_id,
              to_mail: filterParam.to_mail,
              snid: filterParam.snid,
            }}
            resolver={zodResolver(MailListFilterFormSchema) as any}
            onSubmit={(data) => {
              // zod schema 已经处理了类型转换和空值清理
              const transformedData = data as {
                status?: number;
                tpl_id?: string;
                to_mail?: string;
                snid?: string;
              };
              searchGo({
                status: transformedData.status,
                tpl_id: transformedData.tpl_id,
                to_mail: transformedData.to_mail,
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
                tpl_id: undefined,
                to_mail: undefined,
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
                {dictData.mail_send_status && (
                  <FilterDictSelect
                    name="status"
                    placeholder="选择状态"
                    label="状态"
                    disabled={isLoading}
                    dictData={dictData.mail_send_status}
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

                {/* 收件人邮箱过滤 */}
                <FilterInput
                  name="to_mail"
                  placeholder="输入收件人邮箱"
                  label="收件人邮箱"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                />

                {/* 模板ID过滤 */}
                <FilterInput
                  name="tpl_id"
                  placeholder="输入模板ID"
                  label="模板ID"
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

        {/* 表格和分页容器 - 确保不超出页面高度 */}
        <div className="flex-1 flex flex-col min-h-0">
          {/* 数据表格 - 使用 flex-1 但不设置 min-h-0，让分页有足够空间 */}
          <div className="flex-1 overflow-hidden">
            <DataTable
              data={messages}
              columns={columns}
              loading={isLoading}
              error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}

              scrollSnapDelay={300}
              leftStickyColumns={[
                { column: 0, minWidth: "160px", maxWidth: "160px" },
              ]}
              className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
              tableContainerClassName="h-full"
            />
          </div>

          {/* 分页控件 */}
          <div className="flex-shrink-0 pt-4 pb-4">
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

        {/* 日志抽屉 */}
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
            statusClass={mailStatus.getClass(detailMessage.status)}
            statusText={mailStatus.getText(detailMessage.status)}
            dictData={dictData}
          />
        )}
      </div>
    </>
  );
}
