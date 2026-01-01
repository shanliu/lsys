import { userSenderMailerMessageView, type UserSenderMailerMessageItemType } from '@shared/apis/user/sender-mailer';
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading';
import { Badge } from '@shared/components/ui/badge';
import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer';
import { useToast } from '@shared/contexts/toast-context';
import { type TypedDictData } from '@apps/main/hooks/use-dict-data';
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils';
import { createStatusMapper } from '@apps/main/lib/status-utils';
import { createCopyWithToast } from '@shared/lib/utils/copy-utils';
import { useQuery } from '@tanstack/react-query';
import { Copy } from 'lucide-react';

interface ListContentDrawerProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    message: UserSenderMailerMessageItemType;
    statusClass?: string;
    statusText?: string;
    dictData: TypedDictData<['user_sender_mailer']>;
}

export function ListContentDrawer({
    open,
    onOpenChange,
    message,
    statusClass,
    statusText,
    dictData,
}: ListContentDrawerProps) {
    const { success: showSuccess, error: showError } = useToast();
    const copyText = createCopyWithToast(showSuccess, showError);

    // 批次状态映射
    const bodyStatus = createStatusMapper(
        {
            2: 'success',  // 已完成
            1: 'info',     // 待发送
        },
        (status) => dictData.mail_body_status?.getLabel(String(status)) || '发送中'
    );

    // 内容加载
    const contentQuery = useQuery({
        queryKey: ['userSenderMailerMessageView', message.id],
        queryFn: async ({ signal }) => {
            return await userSenderMailerMessageView({ message_id: message.id }, { signal });
        },
        enabled: !!open,
        staleTime: 5 * 60 * 1000,
    });

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader className={cn("pb-6")}>
                    <DrawerTitle className={cn("text-xl")}>邮件详细信息</DrawerTitle>
                </DrawerHeader>

                <div className="space-y-6">
                    {/* 基本信息块 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">基本信息</h3>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">消息ID</span>
                                <span className="text-sm font-medium">{message.snid || '-'}</span>
                            </div>

                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">收件人</span>
                                <span className="text-sm font-medium">{message.to_mail || '-'}</span>
                            </div>

                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">模板</span>
                                <span className="text-sm font-medium">{message.tpl_key || '-'}</span>
                            </div>

                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">发送状态</span>
                                <div>
                                    <Badge className={cn(statusClass)}>{statusText}</Badge>
                                </div>
                            </div>

                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">批次状态</span>
                                <div>
                                    <Badge className={cn(bodyStatus.getClass(message.body_status ?? 0))}>
                                        {bodyStatus.getText(message.body_status ?? 0)}
                                    </Badge>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div className="border-t" />

                    {/* 时间信息块 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">时间信息</h3>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">添加时间</span>
                                <span className="text-sm font-medium">
                                    {message.add_time ? formatTime(message.add_time, TIME_STYLE.RELATIVE_ELEMENT) : '-'}
                                </span>
                            </div>

                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">预期发送时间</span>
                                <span className="text-sm font-medium">
                                    {message.expected_time ? formatTime(message.expected_time, TIME_STYLE.ABSOLUTE_ELEMENT) : '-'}
                                </span>
                            </div>

                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">实际发送时间</span>
                                <span className="text-sm font-medium">
                                    {message.send_time ? formatTime(message.send_time, TIME_STYLE.ABSOLUTE_ELEMENT) : '-'}
                                </span>
                            </div>
                        </div>
                    </div>

                    <div className="border-t" />

                    {/* 发送信息块 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">发送信息</h3>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">尝试次数</span>
                                <span className="text-sm font-medium">{message.try_num} / {message.max_try_num}</span>
                            </div>
                        </div>
                    </div>

                    <div className="border-t" />

                    {/* 内容块 */}
                    <div className="space-y-4">
                        <div className="flex items-center justify-between">
                            <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">发送内容</h3>
                            {!contentQuery.isLoading && !contentQuery.isError && (
                                <button
                                    className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
                                    onClick={() => {
                                        const content = contentQuery.data?.response?.body || '';
                                        copyText(content, '内容已复制');
                                    }}
                                >
                                    <Copy className="h-3.5 w-3.5" />
                                    <span>复制</span>
                                </button>
                            )}
                        </div>
                        {contentQuery.isLoading ? (
                            <CenteredLoading variant="content" />
                        ) : contentQuery.isError ? (
                            <CenteredError variant="content" error={contentQuery.error} onReset={() => contentQuery.refetch()} />
                        ) : (
                            <div className="p-4 border rounded bg-muted/20 text-sm font-mono leading-relaxed whitespace-pre-wrap break-words">
                                {contentQuery.data?.response?.body || '无内容'}
                            </div>
                        )}
                    </div>
                </div>
            </DrawerContent>
        </Drawer>
    );
}
