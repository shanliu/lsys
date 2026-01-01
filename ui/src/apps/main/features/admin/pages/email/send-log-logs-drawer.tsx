import {
  systemSenderMailerMessageLogs,
  type SystemSenderMailerMessageLogItemSchema,
  type SystemSenderMailerMessageItemType,
} from "@shared/apis/admin/sender-mailer";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading";
import { PagePagination, useCountNumManager } from "@apps/main/lib/pagination-utils";
import { Badge } from "@shared/components/ui/badge";
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from "@apps/main/components/local/drawer";
import { formatTime, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import type { DictList } from "@shared/types/apis-dict";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useMemo, useState } from "react";
import type { z } from "zod";

type SystemSenderMailerMessageLogItemType = z.infer<typeof SystemSenderMailerMessageLogItemSchema>;

interface SendLogLogsDrawerProps {
  message: SystemSenderMailerMessageItemType;
  open: boolean;
  onClose: () => void;
  logTypeDict?: DictList;
  logStatusDict?: DictList;
}

export function SendLogLogsDrawer({
  message,
  open,
  onClose,
  logTypeDict,
  logStatusDict,
}: SendLogLogsDrawerProps) {
  const queryClient = useQueryClient();
  // 分页状态
  const [currentPage, setCurrentPage] = useState(1);
  const pageSize = 10;

  // count_num 管理器
  const countNumManager = useCountNumManager({ messageId: message.id });

  // 获取日志列表 - 只有在抽屉打开时才启用查询
  const {
    data: logsData,
    isSuccess,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: ["systemSenderMailerMessageLogs", message.id, currentPage],
    queryFn: ({ signal }) =>
      systemSenderMailerMessageLogs(
        {
          message_id: message.id,
          page: { page: currentPage, limit: pageSize },
          count_num: countNumManager.getCountNum(),
        },
        { signal },
      ),
    enabled: open,
  });

  // 处理 Page 分页查询结果（自动提取 total）
  isSuccess && countNumManager.handlePageQueryResult(logsData);

  const logs = getQueryResponseData<SystemSenderMailerMessageLogItemType[]>(
    logsData,
    [],
  );

  // 刷新数据
  const refreshData = useCallback(() => {
    queryClient.refetchQueries({ queryKey: ["systemSenderMailerMessageLogs", message.id] });
  }, [queryClient, message.id]);

  // 创建日志状态映射器（带背景色）
  const logStatusMapper = useMemo(() => {
    const statusMap: Record<string, 'success' | 'danger' | 'warning'> = {
      '2': 'success',  // 成功
      '3': 'danger',   // 失败
      '5': 'warning',  // 取消
      '6': 'success',  // 回调成功
      '7': 'danger'    // 回调失败
    };
    return createStatusMapper(
      statusMap,
      (status) => logStatusDict?.getLabel(String(status)) || String(status)
    );
  }, [logStatusDict]);

  // 获取日志类型文本
  const getLogTypeText = useCallback((type: string) => {
    return logTypeDict?.getLabel(type) || type;
  }, [logTypeDict]);

  // 重置分页当抽屉打开时
  const handleOpenChange = (open: boolean) => {
    if (!open) {
      setCurrentPage(1); // 关闭时重置分页
    }
    onClose();
  };

  return (
    <Drawer open={open} onOpenChange={handleOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>发送日志</DrawerTitle>
          <DrawerDescription>
            查看邮件 #{message.snid} 的发送日志记录
          </DrawerDescription>
        </DrawerHeader>

        <div className="mt-6 space-y-4 flex flex-col flex-1 min-h-0">
          {/* 日志列表 */}
          <div className="flex-1 overflow-y-auto space-y-3">
            {isLoading ? (
              <CenteredLoading variant="content" iconSize="md" />
            ) : isError ? (
              <CenteredError error={error} variant="content" onReset={refreshData} />
            ) : logs.length === 0 ? (
              <div className="text-center text-muted-foreground py-8">暂无日志</div>
            ) : (
              logs.map((log) => (
                <div key={log.id} className="border rounded-lg p-4 space-y-2 bg-card">
                  {/* 第一行：日志类型和状态 */}
                  <div className="grid grid-cols-2 gap-2">
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs text-muted-foreground">类型:</span>
                      <Badge variant="outline">
                        {getLogTypeText(log.log_type)}
                      </Badge>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs text-muted-foreground">状态:</span>
                      {log.status && (
                        <Badge className={logStatusMapper.getClass(log.status)}>
                          {logStatusMapper.getText(log.status)}
                        </Badge>
                      )}
                    </div>
                  </div>

                  {/* 第二行：发送器类型和执行器类型 */}
                  <div className="grid grid-cols-2 gap-2">
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs text-muted-foreground">发送器:</span>
                      <span className="text-xs">{log.sender_type || '-'}</span>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs text-muted-foreground">执行器:</span>
                      <span className="text-xs">{log.executor_type || '-'}</span>
                    </div>
                  </div>

                  {/* 第三行：创建时间 */}
                  <div className="flex items-center gap-1.5">
                    <span className="text-xs text-muted-foreground">时间:</span>
                    <span className="text-xs">{formatTime(log.create_time, TIME_STYLE.ABSOLUTE_TEXT)}</span>
                  </div>

                  {/* 第四行：日志消息内容 */}
                  <div className="flex gap-1.5">
                    <span className="text-xs text-muted-foreground whitespace-nowrap">内容:</span>
                    <span className="text-xs break-words flex-1">{log.message}</span>
                  </div>
                </div>
              ))
            )}
          </div>

          {/* 分页 */}
          <div className="flex justify-end">
            <PagePagination
              currentPage={currentPage}
              pageSize={pageSize}
              total={countNumManager.getTotal() ?? 0}
              loading={isLoading}
              onChange={(newPage) => setCurrentPage(newPage)}
              showTotal={false}
              showPageSize={false}
              showSizeCount={5}
            />
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  );
}
