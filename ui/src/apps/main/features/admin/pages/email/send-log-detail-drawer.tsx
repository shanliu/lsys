import {
  systemSenderMailerMessageView,
  type SystemSenderMailerMessageItemType,
} from "@shared/apis/admin/sender-mailer";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from "@apps/main/components/local/drawer";
import type { TypedDictData } from "@apps/main/hooks/use-dict-data";
import { cn, formatTime, TIME_STYLE } from "@shared/lib/utils";
import { createStatusMapper, type StatusMapper } from "@apps/main/lib/status-utils";
import { useQuery } from "@tanstack/react-query";

interface SendLogDetailDrawerProps {
  message: SystemSenderMailerMessageItemType;
  open: boolean;
  onClose: () => void;
  dictData: TypedDictData<['admin_sender_mailer']>;
  emailStatus: StatusMapper<number>;
}

export function SendLogDetailDrawer({
  message,
  open,
  onClose,
  dictData,
  emailStatus,
}: SendLogDetailDrawerProps) {
  // 获取邮件内容
  const {
    data: contentData,
    isLoading: contentLoading,
    isError: contentError,
  } = useQuery({
    queryKey: ["systemSenderMailerMessageView", message.id],
    queryFn: ({ signal }) =>
      systemSenderMailerMessageView(
        { message_id: message.id },
        { signal },
      ),
    enabled: open,
  });

  const messageBody = contentData?.response?.body || "";

  // 批次状态映射
  const bodyStatus = createStatusMapper(
    {
      2: "success",
      1: "info",
    },
    (status) => dictData.mail_body_status?.getLabel(String(status)) || "发送中",
  );

  return (
    <Drawer open={open} onOpenChange={onClose}>
      <DrawerContent>
        <DrawerHeader className={cn("space-y-3")}>
          <DrawerTitle>邮件详情</DrawerTitle>
          <DrawerDescription>查看邮件发送详细信息</DrawerDescription>
        </DrawerHeader>

        <div className="mt-6 space-y-6">
          {/* 基本信息 */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-muted-foreground">
              基本信息
            </h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <DetailItem label="消息ID" value={message.snid} />
              <DetailItem label="模板键值">
                <Badge variant="outline">{message.tpl_key}</Badge>
              </DetailItem>
              <DetailItem label="收件人" value={message.to_mail} />
              <DetailItem label="状态">
                <Badge className={emailStatus.getClass(message.status)}>
                  {emailStatus.getText(message.status)}
                </Badge>
              </DetailItem>
              <DetailItem label="批次状态">
                <Badge className={bodyStatus.getClass(message.body_status ?? 0)}>
                  {bodyStatus.getText(message.body_status ?? 0)}
                </Badge>
              </DetailItem>
            </div>
          </div>

          {/* 发送信息 */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-muted-foreground">
              发送信息
            </h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <DetailItem
                label="尝试次数"
                value={`${message.try_num} / ${message.max_try_num}`}
              />

              <DetailItem label="预期发送时间">
                <div className="text-sm">
                  {formatTime(message.expected_time, TIME_STYLE.ABSOLUTE_TEXT)}
                </div>
              </DetailItem>
              <DetailItem label="实际发送时间">
                <div className="text-sm">
                  {message.send_time
                    ? formatTime(message.send_time, TIME_STYLE.ABSOLUTE_TEXT)
                    : "-"}
                </div>
              </DetailItem>
            </div>
          </div>

          {/* 发送内容 */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-muted-foreground">
              发送内容
            </h3>
            {contentLoading ? (
              <div className="p-4 bg-muted/50 rounded-lg">
                <CenteredLoading variant="content" iconSize="sm" className="min-h-[80px]" />
              </div>
            ) : contentError ? (
              <div className="p-4 bg-muted/50 rounded-lg">
                <CenteredError variant="content" error={contentError} className="min-h-[80px]" />
              </div>
            ) : (
              <div className="p-4 bg-muted/50 rounded-lg">
                {messageBody && messageBody !== "-" ? (
                  <CopyableText
                    value={messageBody}
                    message="发送内容已复制"
                    title="点击复制发送内容"
                    showIcon={true}
                    tooltip={true}
                    className="text-xs"
                  />
                ) : (
                  <div className="text-sm text-muted-foreground">-</div>
                )}
              </div>
            )}
          </div>

          {/* 模板参数 */}
          {(message as any).tpl_args && (
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-muted-foreground">
                模板参数
              </h3>
              <div className="p-4 bg-muted/50 rounded-lg">
                <pre className="text-xs overflow-x-auto">
                  {JSON.stringify((message as any).tpl_args, null, 2)}
                </pre>
              </div>
            </div>
          )}

          {/* 错误信息 */}
          {(message as any).err_msg && (
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-muted-foreground">
                错误信息
              </h3>
              <div className="p-4 bg-destructive/10 rounded-lg">
                <p className="text-sm text-destructive">{(message as any).err_msg}</p>
              </div>
            </div>
          )}
        </div>
      </DrawerContent>
    </Drawer>
  );
}

// 详情项组件
function DetailItem({
  label,
  value,
  children,
}: {
  label: string;
  value?: string | number;
  children?: React.ReactNode;
}) {
  return (
    <div className="space-y-1">
      <div className="text-xs text-muted-foreground">{label}</div>
      {children || (
        <div className={cn("text-sm font-medium")}>{value || "-"}</div>
      )}
    </div>
  );
}
