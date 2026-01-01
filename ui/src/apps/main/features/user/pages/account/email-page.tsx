import { emailDelete, emailListData } from '@shared/apis/user/profile';
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog';
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { PageSkeletonTable } from '@shared/components/custom/page-placeholder/skeleton-table';
import { Badge } from '@shared/components/ui/badge';
import { Button } from '@shared/components/ui/button';
import { useToast } from '@shared/contexts/toast-context';
import { useDictData, type TypedDictData } from '@apps/main/hooks/use-dict-data';
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils';
import { createStatusMapper } from '@apps/main/lib/status-utils';
import { formatServerError } from '@shared/lib/utils/format-utils';
import { getQueryResponseData } from '@shared/lib/utils';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Mail, Plus, ShieldCheck, Trash2 } from 'lucide-react';
import React, { useState } from 'react';
import { EmailFormDrawer,  } from './email-form-drawer';
import type { EmailData } from './email-schema';
import { EmailVerifyDrawer,  } from './email-verify-drawer';

interface EmailPageContentProps {
  dictData: TypedDictData<['user_account']>;
}

const EmailPageContent: React.FC<EmailPageContentProps> = ({ dictData }) => {
  const { error } = useToast();
  const queryClient = useQueryClient();

  // 表单状态
  const [isAddSheetOpen, setIsAddSheetOpen] = useState(false);
  const [isVerifySheetOpen, setIsVerifySheetOpen] = useState(false);
  const [verifyingEmail, setVerifyingEmail] = useState<EmailData | null>(null);

  // 获取邮箱列表
  const { data: emailData, isLoading, error: queryError } = useQuery({
    queryKey: ['emailList'],
    queryFn: () => emailListData({})
  });

  const emails = getQueryResponseData<EmailData[]>(emailData, []);

  // 删除邮箱
  const deleteEmailMutation = useMutation({
    mutationFn: (emailId: number) => emailDelete({ email_id: emailId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['emailList'] });
    },
    onError: (err: any) => {
      const errorMessage = formatServerError(err);
      error(errorMessage);
    },
  });

  // 创建状态映射器
  const emailStatus = createStatusMapper(
    {
      1: 'warning',
      2: 'success',
    },
    (status: number) => dictData.email_status.getLabel(String(status)) || '未知'
  );

  // 打开添加邮箱
  const openAddSheet = () => {
    setIsAddSheetOpen(true);
  };

  // 打开验证邮箱
  const openVerifySheet = (email: EmailData) => {
    setVerifyingEmail(email);
    setIsVerifySheetOpen(true);
  };

  // 关闭抽屉
  const closeAddSheet = () => {
    setIsAddSheetOpen(false);
  };

  const closeVerifySheet = () => {
    setIsVerifySheetOpen(false);
    setVerifyingEmail(null);
  };

  // 表单提交成功后回调
  const handleAddSuccess = () => {
    queryClient.invalidateQueries({ queryKey: ['emailList'] });
  };

  const handleVerifySuccess = () => {
    queryClient.invalidateQueries({ queryKey: ['emailList'] });
  };

  // 删除邮箱
  const handleDeleteEmail = (emailId: number) => {
    deleteEmailMutation.mutate(emailId);
  };

  return (
    <div className="px-0 py-6 space-y-6">
      {/* 页面标题和添加按钮 */}
      <div className="flex items-center justify-between px-4 md:px-6">
        <h1 className="text-lg md:text-2xl font-semibold text-foreground">邮箱管理</h1>
        <Button onClick={openAddSheet} size="sm" variant="outline">
          <Plus className="w-4 h-4 mr-2" />
          添加邮箱
        </Button>
      </div>

      {/* 邮箱列表内容 */}
      {isLoading ? (
        <PageSkeletonTable variant="content" rows={3} className={cn('px-4 md:px-6')} />
      ) : queryError ? (
        <CenteredError
          variant="card"
          error={queryError}
          onReset={() => queryClient.refetchQueries({ queryKey: ['emailList'] })}
          className={cn('md:m-6')}
        />
      ) : emails.length > 0 ? (
        <div className={cn("px-4 md:px-6", emails.length === 1 ? "space-y-4" : "grid grid-cols-1 2xl:grid-cols-2 gap-4")}>
          {emails.map((email: EmailData) => (
            <div key={email.id} className="border rounded-lg p-4 hover:shadow-md transition-shadow border-border">
              <div className="flex flex-col md:flex-row md:justify-between md:items-start">
                <div className="flex-1 space-y-2">
                  <div className="flex items-center gap-2">
                    <Mail className="w-4 h-4 text-muted-foreground" />
                    <span className="font-medium text-foreground">{email.email}</span>
                    <Badge className={cn(emailStatus.getClass(email.status))}>
                      {emailStatus.getText(email.status)}
                    </Badge>
                  </div>
                  <div className="text-xs text-muted-foreground ml-6">
                    {email.status === 2
                      ? `确认时间: ${formatTime(email.confirm_time, TIME_STYLE.ABSOLUTE_TEXT)}`
                      : `最后修改: ${formatTime(email.change_time, TIME_STYLE.RELATIVE_TEXT)}`
                    }
                  </div>
                </div>
                <div className="flex items-center justify-end gap-2 mt-4 md:mt-0 md:ml-4">
                  {email.status === 1 && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => openVerifySheet(email)}
                      className={cn("border-border text-foreground hover:bg-muted")}
                    >
                      <ShieldCheck className="w-4 h-4 mr-2" />
                      验证
                    </Button>
                  )}
                  <ConfirmDialog
                    title="删除邮箱"
                    description={`确定要删除邮箱 "${email.email}" 吗？此操作无法撤销。`}
                    onConfirm={() => handleDeleteEmail(email.id)}
                  >
                    <Button variant="outline" size="sm" className={cn("border-border text-foreground hover:bg-muted")}>
                      <Trash2 className="w-4 h-4 mr-2" />
                      删除
                    </Button>
                  </ConfirmDialog>
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="text-center py-8 px-4 md:px-6">
          <Mail className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground mb-4">暂无邮箱信息</p>
          <Button onClick={openAddSheet} variant="outline">
            <Plus className="w-4 h-4 mr-2" />
            添加邮箱
          </Button>
        </div>
      )}

      <EmailFormDrawer
        isOpen={isAddSheetOpen}
        onClose={closeAddSheet}
        onSuccess={handleAddSuccess}
      />

      <EmailVerifyDrawer
        isOpen={isVerifySheetOpen}
        onClose={closeVerifySheet}
        email={verifyingEmail}
        onSuccess={handleVerifySuccess}
      />
    </div>
  );
};

export const ProfileEmailPage: React.FC = () => {
  const queryClient = useQueryClient();

  // 获取字典数据
  const { dictData, isLoading: dictLoading, isError: dictError, errors: dictErrors } = useDictData(['user_account'] as const);

  // 如果字典加载中，显示加载状态
  if (dictLoading) {
    return <PageSkeletonTable variant="table" rows={3} className={cn('px-4 md:px-6')} />;
  }

  // 如果字典加载失败，显示错误页面
  if (dictError && dictErrors.length > 0) {
    return (
      <div className="px-0 py-6 space-y-6">
        <div className="flex items-center justify-between px-4 md:px-6">
          <h1 className="text-lg md:text-2xl font-semibold text-foreground">邮箱管理</h1>
          <Button onClick={() => { }} size="sm" className={cn("bg-primary text-primary-foreground hover:bg-primary/90")} disabled>
            <Plus className="w-4 h-4 mr-2" />
            添加邮箱
          </Button>
        </div>
        <CenteredError
          variant="inline"
          error={dictErrors}
          onReset={() => queryClient.refetchQueries({ queryKey: ['emailList'] })}
        />
      </div>
    );
  }

  return <EmailPageContent dictData={dictData} />;
};
