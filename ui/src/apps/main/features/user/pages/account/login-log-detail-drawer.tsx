import {
  Drawer,
  DrawerContent,
  DrawerHeader,
  DrawerTitle,
} from "@apps/main/components/local/drawer";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import type { AccountLoginHistoryItemType } from "@shared/apis/user/account";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import {
  cn,
  formatTime,
  TIME_STYLE,
} from "@shared/lib/utils";

interface LoginLogDetailDrawerProps {
  log: AccountLoginHistoryItemType | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  loginDictData: any;
}

export function LoginLogDetailDrawer({
  log,
  open,
  onOpenChange,
  loginDictData,
}: LoginLogDetailDrawerProps) {
  if (!log) return null;

  // 创建登录状态映射器（与主页面保持一致）
  const loginStatus = createStatusMapper(
    {
      0: "danger",    // 失败
      1: "warning",   // 仅预登陆
      2: "success",   // 成功
    },
    (status) =>
      loginDictData.login_status?.getLabel(String(status)) || String(status),
  );

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>登录日志详情 #{log.id}</DrawerTitle>
        </DrawerHeader>
        <div className="space-y-6 mt-6">
          <div className="grid grid-cols-2 gap-6">
            <div>
              <label className="text-sm font-medium text-muted-foreground">
                登录状态
              </label>
              <div className="mt-1">
                <Badge variant={undefined} className={cn(loginStatus.getClass(log.is_login))}>
                  {loginStatus.getText(log.is_login)}
                </Badge>
              </div>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">
                登录时间
              </label>
              <div className="mt-1 text-sm">
                {log.add_time
                  ? formatTime(log.add_time, TIME_STYLE.RELATIVE_ELEMENT)
                  : "未知时间"}
              </div>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">
                登录账号
              </label>
              <div className="mt-1">
                <CopyableText value={log.login_account} className={cn("text-sm")} />
              </div>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">
                账号ID
              </label>
              <div className="mt-1 text-sm">{log.account_id}</div>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">
                登录类型
              </label>
              <div className="mt-1">
                <Badge variant="secondary">
                  {loginDictData.login_type?.getLabel(log.login_type) ||
                    log.login_type}
                </Badge>
              </div>
            </div>
            <div>
              <label className="text-sm font-medium text-muted-foreground">
                登录IP
              </label>
              <div className="mt-1">
                <CopyableText value={log.login_ip} className={cn("text-sm")} />
              </div>
            </div>
            {log.login_city && (
              <div>
                <label className="text-sm font-medium text-muted-foreground">
                  登录城市
                </label>
                <div className="mt-1 text-sm">{log.login_city}</div>
              </div>
            )}
          </div>

          {log.login_msg && (
            <div>
              <label className="text-sm font-medium text-muted-foreground">
                登录消息
              </label>
              <div className="mt-1 p-3 bg-muted rounded-md text-sm">
                {log.login_msg}
              </div>
            </div>
          )}
        </div>
      </DrawerContent>
    </Drawer>
  );
}
