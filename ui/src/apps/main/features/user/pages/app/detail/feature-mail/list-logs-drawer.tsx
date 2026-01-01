import { type UserSenderMailerMessageItemType, type UserSenderMailerMessageLogItemType, userSenderMailerMessageLogs } from '@shared/apis/user/sender-mailer'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { PagePagination, useCountNumManager } from '@apps/main/lib/pagination-utils'
import { MaskedText } from '@shared/components/custom/text/masked-text'
import { Badge } from '@shared/components/ui/badge'
import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer'
import { type TypedDictData } from '@apps/main/hooks/use-dict-data'
import { cn, formatTime, getQueryResponseData, TIME_STYLE } from '@shared/lib/utils'
import { createStatusMapper } from '@apps/main/lib/status-utils'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useMemo, useState } from 'react'

interface ListLogsDrawerProps {
  message: UserSenderMailerMessageItemType
  isOpen: boolean
  onOpenChange: (open: boolean) => void
  dictData: TypedDictData<['user_sender_mailer']>
}

export function ListLogsDrawer({
  message,
  isOpen,
  onOpenChange,
  dictData
}: ListLogsDrawerProps) {
  const queryClient = useQueryClient();
  // 分页状态
  const [page, setPage] = useState(1)
  const pageSize = 10

  // count_num 优化管理器
  const countNumManager = useCountNumManager({});

  // 获取日志数据 - 只有在抽屉打开时才启用查询
  const { data: logsData, isSuccess, isLoading, isError, error } = useQuery({
    queryKey: ['userSenderMailerMessageLogs', message.id, page],
    queryFn: ({ signal }) => userSenderMailerMessageLogs({
      message_id: message.id,
      page: {
        page: page,
        limit: pageSize,
      },
      count_num: countNumManager.getCountNum(),
    }, { signal }),
    enabled: isOpen, // 只有在抽屉打开时才查询
  })

  // 处理 Page 分页查询结果（自动提取 total）
  isSuccess && countNumManager.handlePageQueryResult(logsData);

  const logs = getQueryResponseData<UserSenderMailerMessageLogItemType[]>(logsData, [])

  // 刷新数据
  const refreshData = useCallback(() => {
    queryClient.refetchQueries({ queryKey: ['userSenderMailerMessageLogs', message.id] });
  }, [queryClient, message.id]);

  // 创建日志状态映射器（带背景色）
  const logStatusMapper = useMemo(() => createStatusMapper(
    {
      2: 'success',  // 成功
      3: 'danger',   // 失败
      5: 'warning',  // 取消
      6: 'success',  // 回调成功
      7: 'danger'    // 回调失败
    },
    (status) => dictData.log_status.getLabel(String(status)) || String(status)
  ), [dictData.log_status])

  // 获取日志类型文本
  const getLogTypeText = useCallback((type: number) => {
    return dictData.log_type.getLabel(String(type)) || String(type)
  }, [dictData.log_type])

  // 重置分页当抽屉关闭时
  const handleOpenChange = (open: boolean) => {
    onOpenChange(open)
    if (!open) {
      setPage(1) // 关闭时重置分页
    }
  }

  return (
    <Drawer open={isOpen} onOpenChange={handleOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>邮件发送日志</DrawerTitle>
          <DrawerDescription className={cn("space-y-1")}>
            <div>消息ID: {message.snid}</div>
            <div className="flex items-center gap-1.5">
              <span>收件人:</span>
              <MaskedText
                text={message.to_mail}
                type="email"
                clickable={true}
                className="inline"
              />
            </div>
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
                  {/* 第一行：类型和状态 */}
                  <div className="grid grid-cols-2 gap-2">
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs text-muted-foreground">类型:</span>
                      <span className="text-xs font-medium">{getLogTypeText(log.log_type)}</span>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs text-muted-foreground">状态:</span>
                      <Badge className={logStatusMapper.getClass(log.status)}>
                        {logStatusMapper.getText(log.status)}
                      </Badge>
                    </div>
                  </div>

                  {/* 第二行：配置和时间 */}
                  <div className="grid grid-cols-2 gap-2">
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs text-muted-foreground">配置:</span>
                      <span className="text-xs">{log.executor_type || '-'}</span>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <span className="text-xs text-muted-foreground">时间:</span>
                      <span className="text-xs">{formatTime(log.create_time, TIME_STYLE.ABSOLUTE_TEXT)}</span>
                    </div>
                  </div>

                  {/* 第三行：日志内容 */}
                  <div className="flex gap-1.5">
                    <span className="text-xs text-muted-foreground whitespace-nowrap">日志内容:</span>
                    <span className="text-xs break-words flex-1">{log.message}</span>
                  </div>
                </div>
              ))
            )}
          </div>

          {/* 分页 */}
          <div className="flex justify-end">
            <PagePagination
              currentPage={page}
              pageSize={pageSize}
              total={countNumManager.getTotal() ?? 0}
              loading={isLoading}
              onChange={(newPage) => setPage(newPage)}
              showTotal={false}
              showPageSize={false}
              showSizeCount={5}
            />
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  )
}
