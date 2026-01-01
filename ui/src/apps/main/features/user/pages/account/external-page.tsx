import { externalDelete, externalListData } from '@shared/apis/user/profile';
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog';
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { PageSkeletonTable } from '@shared/components/custom/page-placeholder/skeleton-table';
import { Avatar, AvatarFallback, AvatarImage } from '@shared/components/ui/avatar';
import { Badge } from '@shared/components/ui/badge';
import { Button } from '@shared/components/ui/button';
import { useToast } from '@shared/contexts/toast-context';
import { useDictData } from '@apps/main/hooks/use-dict-data';
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils';
import { formatServerError } from '@shared/lib/utils/format-utils';
import { getQueryResponseData } from '@shared/lib/utils';
import { DictList } from '@shared/types/apis-dict';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link2, Plus, Trash2, User } from 'lucide-react';
import React, { useState } from 'react';
import { ExternalBindDrawer } from './external-bind-drawer';
import type { ExternalAccountData } from './external-schema';

// 页面内容组件
interface ExternalPageContentProps {
  exterType: DictList;
}

const ExternalPageContent: React.FC<ExternalPageContentProps> = ({ exterType }) => {
  const { error } = useToast();
  const queryClient = useQueryClient();

  // Sheet 状态
  const [isBindSheetOpen, setIsBindSheetOpen] = useState(false);

  // 获取外部账号列表
  const { data: externalData, isLoading, error: queryError } = useQuery({
    queryKey: ['externalList'],
    queryFn: () => externalListData({})
  });

  const externalAccounts = getQueryResponseData<ExternalAccountData[]>(externalData, []);

  // 删除外部账号
  const deleteExternalMutation = useMutation({
    mutationFn: (extId: number) => externalDelete({ ext_id: extId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['externalList'] });
    },
    onError: (err: any) => {
      const errorMessage = formatServerError(err);
      error(errorMessage);
    },
  });

  // 删除处理
  const handleDelete = (extId: number) => {
    deleteExternalMutation.mutate(extId);
  };

  const openBindSheet = () => {
    setIsBindSheetOpen(true);
  };

  const closeBindSheet = () => {
    setIsBindSheetOpen(false);
  };

  return (
    <div className="px-0 py-6 space-y-6">
      {/* 页面标题和添加按钮 */}
      <div className="flex items-center justify-between px-4 md:px-6">
        <h1 className="text-lg md:text-2xl font-semibold text-foreground">外部账号</h1>
        <Button onClick={openBindSheet} size="sm" variant="outline">
          <Plus className="w-4 h-4 mr-2" />
          绑定账号
        </Button>
      </div>

      {/* 列表内容 */}
      {isLoading ? (
        <PageSkeletonTable variant="content" rows={3} className={cn('px-4 md:px-6')} />
      ) : queryError ? (
        <CenteredError
          className={cn('px-4 md:p-6')}
          variant="card"
          error={queryError}
          onReset={() => queryClient.refetchQueries({ queryKey: ['externalList'] })}
        />
      ) : externalAccounts.length > 0 ? (
        <div className={cn("px-4 md:p-6", externalAccounts.length === 1 ? "space-y-4" : "grid grid-cols-1 2xl:grid-cols-2 gap-4")}>
          {externalAccounts.map((account) => (
            <div key={account.id} className="border rounded-lg p-4 hover:shadow-md transition-shadow border-border">
              <div className="flex flex-col md:flex-row md:justify-between md:items-start">
                <div className="flex items-start gap-4">
                  <Avatar className="h-12 w-12">
                    <AvatarImage src={account.external_pic} alt={account.external_name} />
                    <AvatarFallback><User className="h-6 w-6" /></AvatarFallback>
                  </Avatar>

                  <div className="space-y-1">
                    <div className="flex items-center gap-2 flex-wrap">
                      <span className="font-semibold text-foreground">{account.external_name}</span>
                      <Badge variant="outline" className="text-muted-foreground flex-shrink-0">
                        {exterType.getLabel(account.external_type)}
                      </Badge>
                      {account.status === 1 ? (
                        <Badge className="bg-green-500 hover:bg-green-600 flex-shrink-0">已绑定</Badge>
                      ) : (
                        <Badge variant="secondary" className="flex-shrink-0">未验证</Badge>
                      )}
                    </div>

                    <div className="text-sm text-muted-foreground flex items-center gap-1">
                      <span className="truncate max-w-[200px]" title={account.external_nikename || account.external_name}>
                        {account.external_nikename || account.external_name}
                      </span>
                    </div>

                    <div className="text-xs text-muted-foreground mt-1">
                      绑定时间: {formatTime(account.change_time, TIME_STYLE.ABSOLUTE_TEXT)}
                    </div>
                  </div>
                </div>

                <div className="flex items-center justify-end gap-2 mt-4 md:mt-0 md:ml-4">
                  {account.external_link && (
                    <Button
                      variant="ghost"
                      size="icon"
                      asChild
                      className="h-8 w-8"
                      title="查看外部链接"
                    >
                      <a href={account.external_link} target="_blank" rel="noopener noreferrer">
                        <Link2 className="h-4 w-4" />
                      </a>
                    </Button>
                  )}

                  <ConfirmDialog
                    title="解除绑定"
                    description={`确定要解除与 "${account.external_name}" 的绑定吗？`}
                    onConfirm={() => handleDelete(account.id)}
                  >
                    <Button variant="outline" size="sm" className={cn("border-border text-foreground hover:bg-muted")}>
                      <Trash2 className="w-4 h-4 mr-2" />
                      解绑
                    </Button>
                  </ConfirmDialog>
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="text-center py-8 px-4 md:px-6">
          <User className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground mb-4">暂无外部账号绑定</p>
          <Button onClick={openBindSheet} className={cn("bg-primary text-primary-foreground hover:bg-primary/90")}>
            <Plus className="w-4 h-4 mr-2" />
            绑定账号
          </Button>
        </div>
      )}

      <ExternalBindDrawer
        isOpen={isBindSheetOpen}
        onClose={closeBindSheet}
        availableTypes={[...exterType]}
      />
    </div>
  );
};

export const ProfileExternalPage: React.FC = () => {
  // 使用字典 Hook 获取外部登录类型
  const { dictData, isLoading, isError, refetch, errors } = useDictData(['auth_login'] as const);
  const exterType = dictData.exter_type || new DictList();

  if (isLoading) {
    return <PageSkeletonTable variant="content" rows={3} className={cn('px-4 md:px-6')} />;
  }

  if (isError) {
    return (
      <CenteredError
        variant="card"
        error={errors}
        onReset={refetch}
      />
    );
  }

  return (
    <ExternalPageContent exterType={exterType} />
  );
};
