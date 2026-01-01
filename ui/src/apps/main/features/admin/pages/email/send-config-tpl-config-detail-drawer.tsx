"use client";

import type { SystemSenderMailerTplConfigItemType } from "@shared/apis/admin/sender-mailer";
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from "@apps/main/components/local/drawer";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import { cn, formatTime, TIME_STYLE } from "@shared/lib/utils";

interface EmailSendConfigTplConfigDetailDrawerProps {
  config: SystemSenderMailerTplConfigItemType | null;
  open: boolean;
  onClose: () => void;
}

export function EmailSendConfigTplConfigDetailDrawer({
  config,
  open,
  onClose,
}: EmailSendConfigTplConfigDetailDrawerProps) {
  if (!config) return null;

  return (
    <Drawer open={open} onOpenChange={onClose}>
      <DrawerContent>
        <DrawerHeader className={cn("space-y-3")}>
          <DrawerTitle>邮件模板配置详情</DrawerTitle>
          <DrawerDescription>
            查看配置 <strong>{config.name}</strong> 的详细信息
          </DrawerDescription>
        </DrawerHeader>

        <div className="mt-6 space-y-6">
          {/* 基本信息 */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-muted-foreground">
              基本信息
            </h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <DetailItem label="配置ID" value={config.id} />
              <DetailItem label="配置名称" value={config.name} />
              <DetailItem label="模板Key">
                <CopyableText
                  value={config.tpl_key}
                  message="模板Key已复制"
                  showIcon={true}
                  tooltip={true}
                  className="text-sm font-mono"
                />
              </DetailItem>
              <DetailItem label="应用信息">
                <div className="text-sm">
                  <div className="font-medium">{config.app_name || "-"}</div>
                  <div className="text-xs text-muted-foreground">
                    App ID: {config.app_id}
                  </div>
                </div>
              </DetailItem>
            </div>
          </div>

          {/* SMTP 配置信息 */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-muted-foreground">
              SMTP 配置
            </h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <DetailItem label="SMTP配置名称">
                <Badge variant="outline">{config.setting_name}</Badge>
              </DetailItem>
              <DetailItem label="SMTP配置Key">
                <CopyableText
                  value={config.setting_key}
                  message="配置Key已复制"
                  showIcon={true}
                  tooltip={true}
                  className="text-sm font-mono"
                />
              </DetailItem>
              <DetailItem label="配置ID" value={config.setting_id} />
            </div>
          </div>

          {/* 邮件配置详情 */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-muted-foreground">
              邮件配置详情
            </h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <DetailItem label="发件人邮箱">
                <CopyableText
                  value={config.config_data.from_email}
                  message="发件人邮箱已复制"
                  showIcon={true}
                  tooltip={true}
                  className="text-sm"
                />
              </DetailItem>
              <DetailItem label="回复邮箱">
                {config.config_data.reply_email ? (
                  <CopyableText
                    value={config.config_data.reply_email}
                    message="回复邮箱已复制"
                    showIcon={true}
                    tooltip={true}
                    className="text-sm"
                  />
                ) : (
                  <span className="text-sm text-muted-foreground">-</span>
                )}
              </DetailItem>
              <DetailItem label="主题模板ID">
                <CopyableText
                  value={config.config_data.subject_tpl_id}
                  message="主题模板ID已复制"
                  showIcon={true}
                  tooltip={true}
                  className="text-sm font-mono"
                />
              </DetailItem>
              <DetailItem label="正文模板ID">
                <CopyableText
                  value={config.config_data.body_tpl_id}
                  message="正文模板ID已复制"
                  showIcon={true}
                  tooltip={true}
                  className="text-sm font-mono"
                />
              </DetailItem>
            </div>
          </div>

          {/* 其他信息 */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-muted-foreground">
              其他信息
            </h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <DetailItem label="创建用户ID" value={config.user_id} />
              <DetailItem label="修改用户ID" value={config.change_user_id} />
              <DetailItem label="修改时间">
                <div className="text-sm">
                  {formatTime(config.change_time, TIME_STYLE.ABSOLUTE_TEXT)}
                </div>
              </DetailItem>
            </div>
          </div>
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
