import type { AppRbacAuditDataItemType, AppRbacAuditDetailItemType } from '@shared/apis/user/rbac';
import { Badge } from '@shared/components/ui/badge';
import {
  Drawer,
  DrawerContent,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer';
import { useToast } from '@shared/contexts/toast-context';
import { useDictData } from '@apps/main/hooks/use-dict-data';
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils';
import { createStatusMapper } from '@apps/main/lib/status-utils';
import { createCopyWithToast } from '@shared/lib/utils/copy-utils';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@shared/components/ui/collapsible';
import { ChevronDown, Copy, Loader2 } from 'lucide-react';
import { ScrollArea } from '@shared/components/ui/scroll-area';
import { useMemo, useState } from 'react';

interface AuditDetailDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  audit: AppRbacAuditDataItemType | null;
}

export function AuditDetailDrawer({
  open,
  onOpenChange,
  audit,
}: AuditDetailDrawerProps) {
  const { success: showSuccess, error: showError } = useToast();
  const copyText = createCopyWithToast(showSuccess, showError);

  // 内部获取字典数据
  const { dictData, isLoading: dictLoading } = useDictData(['app_rbac'] as const);

  // 审计结果状态映射
  const checkResultStatus = useMemo(
    () =>
      createStatusMapper<string>(
        {
          allow: 'success',
          deny: 'danger',
        } as Record<string, 'success' | 'danger'>,
        (result) => dictData.audit_result?.getLabel(String(result)) || String(result)
      ),
    [dictData.audit_result]
  );

  if (!audit) return null;

  const { audit: auditInfo, detail, user } = audit;

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent className="max-h-[90vh]">
        <DrawerHeader className={cn('pb-4')}>
          <DrawerTitle className={cn('text-xl')}>审计详细信息</DrawerTitle>
        </DrawerHeader>

        {dictLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <ScrollArea className="flex-1 overflow-auto">
            <div className="space-y-6 px-1">
              {/* 基本信息块 */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
                基本信息
              </h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="flex flex-col space-y-1">
                  <span className="text-xs text-muted-foreground">审计ID</span>
                  <span className="text-sm font-medium">{auditInfo.id}</span>
                </div>

                <div className="flex flex-col space-y-1">
                  <span className="text-xs text-muted-foreground">审计结果</span>
                  <div>
                    <Badge className={cn(checkResultStatus.getClass(auditInfo.check_result as 'allow' | 'deny'))}>
                      {checkResultStatus.getText(auditInfo.check_result as 'allow' | 'deny')}
                    </Badge>
                  </div>
                </div>

                <div className="flex flex-col space-y-1">
                  <span className="text-xs text-muted-foreground">用户ID</span>
                  <span className="text-sm font-medium">{auditInfo.user_id}</span>
                </div>

                <div className="flex flex-col space-y-1">
                  <span className="text-xs text-muted-foreground">用户应用ID</span>
                  <span className="text-sm font-medium">{auditInfo.user_app_id || '-'}</span>
                </div>

                <div className="flex flex-col space-y-1">
                  <span className="text-xs text-muted-foreground">审计时间</span>
                  <span className="text-sm font-medium">
                    {auditInfo.add_time
                      ? formatTime(auditInfo.add_time, TIME_STYLE.ABSOLUTE_ELEMENT)
                      : '-'}
                  </span>
                </div>
              </div>
            </div>

            <div className="border-t" />

            {/* 用户信息块 */}
            {user && (
              <>
                <div className="space-y-4">
                  <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
                    用户信息
                  </h3>
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div className="flex flex-col space-y-1">
                      <span className="text-xs text-muted-foreground">用户账号</span>
                      <span className="text-sm font-medium">{user.user_account || '-'}</span>
                    </div>

                    <div className="flex flex-col space-y-1">
                      <span className="text-xs text-muted-foreground">用户昵称</span>
                      <span className="text-sm font-medium">{user.user_nickname || '-'}</span>
                    </div>
                  </div>
                </div>
                <div className="border-t" />
              </>
            )}

            {/* 设备信息块 */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
                设备信息
              </h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="flex flex-col space-y-1">
                  <span className="text-xs text-muted-foreground">IP地址</span>
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium">{auditInfo.user_ip || '-'}</span>
                    {auditInfo.user_ip && (
                      <button
                        className="text-muted-foreground hover:text-foreground transition-colors"
                        onClick={() => copyText(auditInfo.user_ip, 'IP已复制')}
                      >
                        <Copy className="h-3 w-3" />
                      </button>
                    )}
                  </div>
                </div>

                <div className="flex flex-col space-y-1">
                  <span className="text-xs text-muted-foreground">设备ID</span>
                  <div className="flex items-center gap-2">
                    <span className="font-medium font-mono text-xs">
                      {auditInfo.device_id || '-'}
                    </span>
                    {auditInfo.device_id && (
                      <button
                        className="text-muted-foreground hover:text-foreground transition-colors"
                        onClick={() => copyText(auditInfo.device_id, '设备ID已复制')}
                      >
                        <Copy className="h-3 w-3" />
                      </button>
                    )}
                  </div>
                </div>

                <div className="flex flex-col space-y-1">
                  <span className="text-xs text-muted-foreground">设备名称</span>
                  <span className="text-sm font-medium">{auditInfo.device_name || '-'}</span>
                </div>

                <div className="flex flex-col space-y-1">
                  <span className="text-xs text-muted-foreground">请求ID</span>
                  <div className="flex items-center gap-2">
                    <span className="font-medium font-mono text-xs truncate max-w-[150px]">
                      {auditInfo.request_id || '-'}
                    </span>
                    {auditInfo.request_id && (
                      <button
                        className="text-muted-foreground hover:text-foreground transition-colors"
                        onClick={() => copyText(auditInfo.request_id, '请求ID已复制')}
                      >
                        <Copy className="h-3 w-3" />
                      </button>
                    )}
                  </div>
                </div>
              </div>
            </div>

            <div className="border-t" />

            {/* 角色信息块 */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
                角色信息
              </h3>
              <div className="space-y-2">
                <div className="flex flex-col space-y-1">
                  <span className="text-xs text-muted-foreground">角色Key数据</span>
                  <code className="text-xs bg-muted p-2 rounded break-all">
                    {auditInfo.role_key_data || '-'}
                  </code>
                </div>

                <div className="flex flex-col space-y-1">
                  <span className="text-xs text-muted-foreground">Token数据</span>
                  <code className="text-xs bg-muted p-2 rounded break-all">
                    {auditInfo.token_data || '-'}
                  </code>
                </div>
              </div>
            </div>

            <div className="border-t" />

            {/* 审计详情列表 */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
                审计详情 ({detail.length})
              </h3>
              {detail.length > 0 ? (
                <div className="space-y-2">
                  {detail.map((item, index) => (
                    <AuditDetailItem
                      key={item.id}
                      item={item}
                      index={index}
                      checkResultStatus={checkResultStatus}
                    />
                  ))}
                </div>
              ) : (
                <div className="text-sm text-muted-foreground text-center py-4">
                  暂无审计详情
                </div>
              )}
            </div>
          </div>
        </ScrollArea>
        )}
      </DrawerContent>
    </Drawer>
  );
}

interface AuditDetailItemProps {
  item: AppRbacAuditDetailItemType;
  index: number;
  checkResultStatus: ReturnType<typeof createStatusMapper<string>>;
}

function AuditDetailItem({ item, index, checkResultStatus }: AuditDetailItemProps) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <Collapsible open={isOpen} onOpenChange={setIsOpen} className="border rounded-lg">
      <CollapsibleTrigger className="flex items-center justify-between w-full p-3 hover:bg-muted/50 transition-colors">
        <div className="flex items-center gap-3 text-left">
          <span className="text-sm font-medium">#{index + 1}</span>
          <code className="text-xs bg-muted px-2 py-0.5 rounded">{item.op_key}</code>
          <Badge className={cn(checkResultStatus.getClass(item.check_result), 'text-xs')}>
            {checkResultStatus.getText(item.check_result)}
          </Badge>
        </div>
        <ChevronDown className={cn("h-4 w-4 transition-transform duration-200", isOpen && "rotate-180")} />
      </CollapsibleTrigger>
      <CollapsibleContent className="px-3 pb-3">
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2 border-t">
          <div className="flex flex-col space-y-1">
            <span className="text-xs text-muted-foreground">操作ID</span>
            <span className="text-sm">{item.op_id}</span>
          </div>

          <div className="flex flex-col space-y-1">
            <span className="text-xs text-muted-foreground">操作Key</span>
            <code className="text-sm bg-muted px-2 py-0.5 rounded w-fit">{item.op_key}</code>
          </div>

          <div className="flex flex-col space-y-1">
            <span className="text-xs text-muted-foreground">资源ID</span>
            <span className="text-sm">{item.res_id}</span>
          </div>

          <div className="flex flex-col space-y-1">
            <span className="text-xs text-muted-foreground">资源类型</span>
            <span className="text-sm">{item.res_type || '-'}</span>
          </div>

          <div className="flex flex-col space-y-1">
            <span className="text-xs text-muted-foreground">资源用户ID</span>
            <span className="text-sm">{item.res_user_id}</span>
          </div>

          <div className="flex flex-col space-y-1">
            <span className="text-xs text-muted-foreground">审计时间</span>
            <span className="text-sm">
              {item.add_time ? formatTime(item.add_time, TIME_STYLE.RELATIVE_ELEMENT) : '-'}
            </span>
          </div>

          <div className="flex flex-col space-y-1 sm:col-span-2">
            <span className="text-xs text-muted-foreground">权限检查</span>
            <div className="flex flex-wrap gap-2 text-xs">
              {item.is_root === '1' && <Badge variant="outline">Root</Badge>}
              {item.is_self === '1' && <Badge variant="outline">Self</Badge>}
              {item.is_role_all === '1' && <Badge variant="outline">Role All</Badge>}
              {item.is_role_include === '1' && <Badge variant="outline">Role Include</Badge>}
              {item.is_role_excluce === '1' && <Badge variant="outline">Role Exclude</Badge>}
              {item.is_root !== '1' &&
                item.is_self !== '1' &&
                item.is_role_all !== '1' &&
                item.is_role_include !== '1' &&
                item.is_role_excluce !== '1' && (
                  <span className="text-muted-foreground">无特殊权限</span>
                )}
            </div>
          </div>

          {item.res_data && (
            <div className="flex flex-col space-y-1 sm:col-span-2">
              <span className="text-xs text-muted-foreground">资源数据</span>
              <code className="text-xs bg-muted p-2 rounded break-all">{item.res_data}</code>
            </div>
          )}

          {item.role_data && (
            <div className="flex flex-col space-y-1 sm:col-span-2">
              <span className="text-xs text-muted-foreground">角色数据</span>
              <code className="text-xs bg-muted p-2 rounded break-all">{item.role_data}</code>
            </div>
          )}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}
