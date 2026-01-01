"use client";

import type { SystemSenderSmsTplConfigItemType } from "@shared/apis/admin/sender-sms";
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

interface SmsSendConfigTplConfigDetailDrawerProps {
  config: SystemSenderSmsTplConfigItemType | null;
  open: boolean;
  onClose: () => void;
}

export function SmsSendConfigTplConfigDetailDrawer({
  config,
  open,
  onClose,
}: SmsSendConfigTplConfigDetailDrawerProps) {
  if (!config) return null;

  return (
    <Drawer open={open} onOpenChange={onClose}>
      <DrawerContent >
        <DrawerHeader>
          <DrawerTitle>短信模板配置详情</DrawerTitle>
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

          {/* 短信服务配置 */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-muted-foreground">
              短信服务配置
            </h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <DetailItem label="配置名称">
                <Badge variant="outline">{config.setting_name}</Badge>
              </DetailItem>
              <DetailItem label="配置Key">
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

          {/* 模板配置详情 */}
          <div className="space-y-4">
            <h3 className="text-sm font-semibold text-muted-foreground">
              模板配置详情
            </h3>
            <div className="grid grid-cols-1 gap-4">
              {config.config_data?.template_id && (
                <DetailItem label="模板ID">
                  <CopyableText
                    value={config.config_data.template_id}
                    message="模板ID已复制"
                    showIcon={true}
                    tooltip={true}
                    className="text-sm font-mono"
                  />
                </DetailItem>
              )}
              {config.config_data?.template_map && (
                <DetailItem label="模板参数映射">
                  <div className="p-3 bg-muted/50 rounded-lg">
                    <CopyableText
                      value={config.config_data.template_map}
                      message="模板参数映射已复制"
                      showIcon={true}
                      tooltip={true}
                      className="text-xs font-mono break-all"
                    />
                  </div>
                </DetailItem>
              )}
              {/* 兼容邮件配置格式 */}
              {config.config_data?.from_email && (
                <DetailItem label="发件人邮箱">
                  <CopyableText
                    value={config.config_data.from_email}
                    message="发件人邮箱已复制"
                    showIcon={true}
                    tooltip={true}
                    className="text-sm"
                  />
                </DetailItem>
              )}
              {config.config_data?.subject_tpl_id && (
                <DetailItem label="主题模板ID">
                  <CopyableText
                    value={config.config_data.subject_tpl_id}
                    message="主题模板ID已复制"
                    showIcon={true}
                    tooltip={true}
                    className="text-sm font-mono"
                  />
                </DetailItem>
              )}
              {config.config_data?.body_tpl_id && (
                <DetailItem label="正文模板ID">
                  <CopyableText
                    value={config.config_data.body_tpl_id}
                    message="正文模板ID已复制"
                    showIcon={true}
                    tooltip={true}
                    className="text-sm font-mono"
                  />
                </DetailItem>
              )}
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
