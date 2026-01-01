import { mobileDelete, mobileListData } from '@shared/apis/user/profile';
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog';
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { PageSkeletonTable } from '@shared/components/custom/page-placeholder/skeleton-table';
import { Badge } from '@shared/components/ui/badge';
import { Button } from '@shared/components/ui/button';
import { useToast } from '@shared/contexts/toast-context';
import { useDictData, type TypedDictData } from '@apps/main/hooks/use-dict-data';
import { cn, formatServerError, formatTime, TIME_STYLE } from '@shared/lib/utils';
import { createStatusMapper } from '@apps/main/lib/status-utils';
import { getQueryResponseData } from '@shared/lib/utils';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Plus, ShieldCheck, Smartphone, Trash2 } from 'lucide-react';
import React, { useState } from 'react';
import type { MobileData } from './mobile-schema';
import { MobileVerifyDrawer } from './mobile-verify-drawer';
import { MobileFormDrawer } from './mobile-form-drawer';

interface MobilePageContentProps {
  dictData: TypedDictData<['user_account']>;
}

const MobilePageContent: React.FC<MobilePageContentProps> = ({ dictData }) => {
  const { error } = useToast();
  const queryClient = useQueryClient();

  // 表单状态
  const [isAddSheetOpen, setIsAddSheetOpen] = useState(false);
  const [isVerifySheetOpen, setIsVerifySheetOpen] = useState(false);
  const [verifyingMobile, setVerifyingMobile] = useState<MobileData | null>(null);

  // 获取手机号列表
  const { data: mobileData, isLoading, error: queryError } = useQuery({
    queryKey: ['mobileList'],
    queryFn: () => mobileListData({})
  });

  const mobiles = getQueryResponseData<MobileData[]>(mobileData, []);

  // 删除手机号
  const deleteMobileMutation = useMutation({
    mutationFn: (mobileId: number) => mobileDelete({ mobile_id: mobileId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mobileList'] });
    },
    onError: (err: any) => {
      const errorMessage = formatServerError(err);
      error(errorMessage);
    },
  });

  // 创建状态映射器
  const mobileStatus = createStatusMapper(
    {
      1: 'warning',
      2: 'success',
    },
    (status: number) => dictData.mobile_status.getLabel(String(status)) || '未知'
  );

  // 事件处理
  const handleAddMobile = () => {
    setIsAddSheetOpen(true);
  };

  const handleVerifyMobile = (mobile: MobileData) => {
    setVerifyingMobile(mobile);
    setIsVerifySheetOpen(true);
  };

  const handleAddSuccess = () => {
    queryClient.invalidateQueries({ queryKey: ['mobileList'] });
  };

  const handleVerifySuccess = () => {
    queryClient.invalidateQueries({ queryKey: ['mobileList'] });
  };

  // 删除手机号
  const handleDeleteMobile = (mobileId: number) => {
    deleteMobileMutation.mutate(mobileId);
  };

  return (
    <div className="px-0 py-6 space-y-6">
      {/* 页面标题和添加按钮 */}
      <div className="flex items-center justify-between px-4 md:px-6">
        <h1 className="text-lg md:text-2xl font-semibold text-foreground">手机号管理</h1>
        <Button onClick={handleAddMobile} size="sm" variant="outline">
          <Plus className="w-4 h-4 mr-2" />
          添加手机号
        </Button>
      </div>

      {/* 手机号列表内容 */}
      {isLoading ? (
        <PageSkeletonTable variant="content" rows={3} className={cn('px-4 md:px-6')} />
      ) : queryError ? (
        <CenteredError
          variant="card"
          error={queryError}
          onReset={() => queryClient.refetchQueries({ queryKey: ['mobileList'] })}
          className={cn('md:m-6')}
        />
      ) : mobiles.length > 0 ? (
        <div className={cn("px-4 md:px-6", mobiles.length === 1 ? "space-y-4" : "grid grid-cols-1 2xl:grid-cols-2 gap-4")}>
          {mobiles.map((mobile: MobileData) => (
            <div key={mobile.id} className="border rounded-lg p-4 hover:shadow-md transition-shadow border-border">
              <div className="flex flex-col md:flex-row md:justify-between md:items-start">
                <div className="flex-1 space-y-2">
                  <div className="flex items-center gap-2">
                    <Smartphone className="w-4 h-4 text-muted-foreground" />
                    <span className="font-medium text-foreground">{mobile.area_code} {mobile.mobile}</span>
                    <Badge className={cn(mobileStatus.getClass(mobile.status))}>
                      {mobileStatus.getText(mobile.status)}
                    </Badge>
                  </div>
                  <div className="text-xs text-muted-foreground ml-6">
                    {mobile.status === 2
                      ? `确认时间: ${formatTime(mobile.confirm_time, TIME_STYLE.ABSOLUTE_TEXT)}`
                      : `最后修改: ${formatTime(mobile.change_time, TIME_STYLE.RELATIVE_TEXT)}`
                    }
                  </div>
                </div>
                <div className="flex items-center justify-end gap-2 mt-4 md:mt-0 md:ml-4">
                  {mobile.status === 1 && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleVerifyMobile(mobile)}
                      className={cn("border-border text-foreground hover:bg-muted")}
                    >
                      <ShieldCheck className="w-4 h-4 mr-2" />
                      验证
                    </Button>
                  )}
                  <ConfirmDialog
                    title="删除手机号"
                    description={`确定要删除手机号 "${mobile.area_code} ${mobile.mobile}" 吗？此操作无法撤销。`}
                    onConfirm={() => handleDeleteMobile(mobile.id)}
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
          <Smartphone className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground mb-4">暂无手机号信息</p>
          <Button onClick={handleAddMobile} variant="outline">
            <Plus className="w-4 h-4 mr-2" />
            添加手机号
          </Button>
        </div>
      )}

      <MobileFormDrawer
        isOpen={isAddSheetOpen}
        onClose={() => setIsAddSheetOpen(false)}
        onSuccess={handleAddSuccess}
      />

      {verifyingMobile && (
        <MobileVerifyDrawer
          isOpen={isVerifySheetOpen}
          onClose={() => setIsVerifySheetOpen(false)}
          mobile={verifyingMobile}
          onSuccess={handleVerifySuccess}
        />
      )}
    </div>
  );
};

export const ProfileMobilePage: React.FC = () => {
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
          <h1 className="text-lg md:text-2xl font-semibold text-foreground">手机号管理</h1>
          <Button onClick={() => { }} size="sm" className={cn("bg-primary text-primary-foreground hover:bg-primary/90")} disabled>
            <Plus className="w-4 h-4 mr-2" />
            添加手机号
          </Button>
        </div>
        <CenteredError
          variant="inline"
          error={dictErrors}
          onReset={() => queryClient.refetchQueries({ queryKey: ['mobileList'] })}
        />
      </div>
    );
  }

  return <MobilePageContent dictData={dictData} />;
};
