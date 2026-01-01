import {
  accountPasswordModify,
  accountPasswordModifyInfo,
  type AccountPasswordModifyInfoResType,
  type AccountPasswordModifyParamType
} from '@shared/apis/user/account';
import { LoadingButton } from '@apps/main/components/local/sender-config/loading-button';
import { PasswordInput } from '@shared/components/custom/input/password-input';
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading';
import { Badge } from '@shared/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@shared/components/ui/card';
import { Label } from '@shared/components/ui/label';
import { useToast } from '@shared/contexts/toast-context';
import { cn } from '@shared/lib/utils';
import { formatServerError, formatTime, TIME_STYLE } from '@shared/lib/utils/format-utils';
import { getQueryResponseData } from '@shared/lib/utils';
import { useMutation, useQuery } from '@tanstack/react-query';
import React from 'react';
import { Controller, useForm } from 'react-hook-form';

interface FormData {
  oldPassword: string;
  newPassword: string;
  confirmPassword: string;
}

export const AccountChangePasswordPage: React.FC = () => {
  const toast = useToast();

  // 初始化表单
  const form = useForm<FormData>({
    defaultValues: {
      oldPassword: '',
      newPassword: '',
      confirmPassword: ''
    }
  });

  const { control, watch, handleSubmit, formState: { errors }, reset } = form;
  const watchedNewPassword = watch('newPassword');
  const watchedConfirmPassword = watch('confirmPassword');

  // 获取密码修改信息
  const { data: passwordData, isLoading, error: queryError, refetch } = useQuery({
    queryKey: ['account-password-modify-info'],
    queryFn: async ({ signal }) => {
      return await accountPasswordModifyInfo({ signal });
    },
    staleTime: 0,
    gcTime: 0,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    retry: false
  });

  const passwordInfo = getQueryResponseData<AccountPasswordModifyInfoResType | undefined>(passwordData, undefined);
  // last_time === 0 表示未设置密码，last_time > 0 表示已设置密码
  const hasPassword = passwordInfo ? (passwordInfo.last_time?.getTime() ?? 0) > 0 : false;

  // 修改密码 - 使用 useMutation
  const passwordModifyMutation = useMutation({
    mutationFn: async (data: FormData) => {
      const param: AccountPasswordModifyParamType = {
        new_password: data.newPassword
      };

      // 如果已有密码，需要提供旧密码
      if (hasPassword) {
        param.old_password = data.oldPassword;
      }

      return await accountPasswordModify(param);
    },
    onSuccess: (result) => {
      if (result.status) {
        toast.success(result.message || (hasPassword ? '密码修改成功' : '密码设置成功'));
        reset();
        // 重新获取密码信息
        refetch();
      } else {
        toast.error(formatServerError(result));
      }
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    }
  });

  // 表单提交处理
  const onSubmit = (data: FormData) => {
    if (data.newPassword !== data.confirmPassword) {
      toast.error('两次输入的密码不一致');
      return;
    }

    passwordModifyMutation.mutate(data);
  };

  if (isLoading) {
    return <CenteredLoading variant="card" className={cn('m-4 md:m-6')} />;
  }

  if (queryError) {
    return (
      <CenteredError
        variant="card"
        error={queryError}
        onReset={() => refetch()}
        className={cn('m-4 md:m-6')}
      />
    );
  }

  // 格式化密码过期时间提醒
  const formatPasswordTimeout = (remainingSeconds: number, totalTimeout?: number) => {
    if (totalTimeout === 0) return '密码永不过期';
    if (remainingSeconds <= 0) return '密码已过期';

    const remainingDays = Math.ceil(remainingSeconds / (24 * 60 * 60));
    const remainingHours = Math.ceil(remainingSeconds / (60 * 60));

    if (remainingSeconds < 60 * 60) { // 小于1小时
      const minutes = Math.ceil(remainingSeconds / 60);
      return `密码将在 ${minutes} 分钟后过期`;
    } else if (remainingSeconds < 24 * 60 * 60) { // 小于1天
      return `密码将在 ${remainingHours} 小时后过期`;
    } else if (remainingDays <= 30) { // 30天内
      return `密码将在 ${remainingDays} 天后过期`;
    } else {
      return `密码有效期还有 ${remainingDays} 天`;
    }
  };

  // 格式化总有效期
  const formatTotalTimeout = (totalSeconds: number) => {
    if (totalSeconds === 0) return '永久有效';
    const days = Math.floor(totalSeconds / (24 * 60 * 60));
    return `${days} 天`;
  };

  // 检查密码是否即将过期（7天内过期或已过期）
  const isPasswordExpiringSoon = (remainingSeconds: number, totalTimeout?: number) => {
    if (totalTimeout === 0) return false; // 永久有效
    return remainingSeconds > 0 && remainingSeconds <= 7 * 24 * 60 * 60; // 7天内过期
  };

  return (
    <div className="px-4 py-6 space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className={cn("flex items-center gap-2")}>
            {hasPassword ? '修改登录密码' : '设置登录密码'}
            <Badge
              variant={
                !hasPassword ? "secondary" :
                  passwordInfo && passwordInfo.is_expired ? "destructive" :
                    passwordInfo && isPasswordExpiringSoon(passwordInfo.remaining_time, passwordInfo.total_timeout) ? "default" :
                      "outline"
              }
              className={cn("text-xs")}
            >
              {!hasPassword ? '未设置密码' :
                passwordInfo && passwordInfo.is_expired ? '密码已过期' :
                  passwordInfo && isPasswordExpiringSoon(passwordInfo.remaining_time, passwordInfo.total_timeout) ? '密码即将过期' :
                    '已设置密码'}
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent className={cn("space-y-6")}>
          {/* 密码状态信息 */}
          {hasPassword && passwordInfo && (
            <div className="space-y-3">
              <div className="p-4 bg-muted/50 rounded-md border">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {passwordInfo.last_time ? <div>
                    <Label className={cn("text-sm font-medium text-muted-foreground")}>
                      上次修改时间
                    </Label>
                    <p className="text-sm mt-1">
                      {formatTime(passwordInfo.last_time, TIME_STYLE.ABSOLUTE_TEXT)}
                    </p>
                  </div> : null}
                  <div>
                    <Label className={cn("text-sm font-medium text-muted-foreground")}>
                      密码有效期
                    </Label>
                    <p className="text-sm mt-1">
                      {formatTotalTimeout(passwordInfo.total_timeout)}
                    </p>
                  </div>
                  <div className="md:col-span-2">
                    <Label className={cn("text-sm font-medium text-muted-foreground")}>
                      密码状态
                    </Label>
                    <div className="flex items-center gap-2 mt-1">
                      <div className={`w-2 h-2 rounded-full ${passwordInfo.is_expired
                        ? 'bg-destructive'
                        : isPasswordExpiringSoon(passwordInfo.remaining_time, passwordInfo.total_timeout)
                          ? 'bg-warning'
                          : 'bg-green-500'
                        }`}></div>
                      <p className={`text-sm ${passwordInfo.is_expired
                        ? 'text-destructive'
                        : isPasswordExpiringSoon(passwordInfo.remaining_time, passwordInfo.total_timeout)
                          ? 'text-warning'
                          : 'text-green-600 dark:text-green-400'
                        }`}>
                        {formatPasswordTimeout(passwordInfo.remaining_time, passwordInfo.total_timeout)}
                      </p>
                    </div>
                  </div>
                </div>
              </div>

              {/* 密码过期警告 */}
              {(passwordInfo.is_expired || isPasswordExpiringSoon(passwordInfo.remaining_time, passwordInfo.total_timeout)) && (
                <div className={`p-4 border rounded-md ${passwordInfo.is_expired
                  ? 'bg-destructive/10 border-destructive/20'
                  : 'bg-warning/10 border-warning/20'
                  }`}>
                  <div className="flex items-start">
                    <div className="flex-shrink-0">
                      <svg className={`h-5 w-5 ${passwordInfo.is_expired
                        ? 'text-destructive'
                        : 'text-warning'
                        }`} viewBox="0 0 20 20" fill="currentColor">
                        <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                      </svg>
                    </div>
                    <div className="ml-3 flex-1">
                      <h3 className={`text-sm font-medium ${passwordInfo.is_expired
                        ? 'text-destructive'
                        : 'text-warning'
                        }`}>
                        {passwordInfo.is_expired
                          ? '密码已过期'
                          : '密码即将过期'}
                      </h3>
                      <div className="mt-1 text-sm text-muted-foreground">
                        <p>{formatPasswordTimeout(passwordInfo.remaining_time, passwordInfo.total_timeout)}</p>
                        <p className="mt-1">
                          {passwordInfo.is_expired
                            ? '您的密码已过期，为确保账号安全，请立即修改密码。'
                            : '为了确保账号安全，建议及时修改密码。'}
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* 设置密码提示 */}
          {!hasPassword && (
            <div className="p-4 bg-primary/10 border border-primary/20 rounded-md">
              <div className="flex items-start">
                <div className="flex-shrink-0">
                  <svg className="h-5 w-5 text-primary" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                  </svg>
                </div>
                <div className="ml-3 flex-1">
                  <h3 className="text-sm font-medium text-primary">
                    设置登录密码
                  </h3>
                  <div className="mt-1 text-sm text-muted-foreground">
                    <p>您的账号尚未设置登录密码，设置密码后可以提高账号安全性。</p>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* 密码修改表单 */}
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            {/* 旧密码输入 - 仅在已有密码时显示 */}
            {hasPassword && (
              <div className="space-y-2">
                <Label htmlFor="old-password" className={cn("text-sm font-medium")}>
                  当前密码
                </Label>
                <Controller
                  name="oldPassword"
                  control={control}
                  rules={{
                    required: '当前密码不能为空'
                  }}
                  render={({ field }) => (
                    <PasswordInput
                      {...field}
                      id="old-password"
                      placeholder="请输入当前密码"
                      className={cn(errors.oldPassword ? 'border-destructive focus:border-destructive' : '')}
                      disabled={passwordModifyMutation.isPending}
                    />
                  )}
                />
                {errors.oldPassword && (
                  <p className="text-sm text-destructive">{errors.oldPassword.message}</p>
                )}
              </div>
            )}

            {/* 新密码输入 */}
            <div className="space-y-2">
              <Label htmlFor="new-password" className={cn("text-sm font-medium")}>
                新密码
              </Label>
              <Controller
                name="newPassword"
                control={control}
                rules={{
                  required: '新密码不能为空',
                  minLength: {
                    value: 6,
                    message: '新密码长度至少6位'
                  }
                }}
                render={({ field }) => (
                  <PasswordInput
                    {...field}
                    id="new-password"
                    placeholder="请输入新密码（至少6位）"
                    className={cn(errors.newPassword ? 'border-destructive focus:border-destructive' : '')}
                    disabled={passwordModifyMutation.isPending}
                  />
                )}
              />
              {errors.newPassword && (
                <p className="text-sm text-destructive">{errors.newPassword.message}</p>
              )}
            </div>

            {/* 确认密码输入 */}
            <div className="space-y-2">
              <Label htmlFor="confirm-password" className={cn("text-sm font-medium")}>
                确认新密码
              </Label>
              <Controller
                name="confirmPassword"
                control={control}
                rules={{
                  required: '请确认新密码',
                  validate: (value) =>
                    value === watchedNewPassword || '两次输入的密码不一致'
                }}
                render={({ field }) => (
                  <PasswordInput
                    {...field}
                    id="confirm-password"
                    placeholder="请再次输入新密码"
                    className={cn(errors.confirmPassword ? 'border-destructive focus:border-destructive' : '')}
                    disabled={passwordModifyMutation.isPending}
                  />
                )}
              />
              {errors.confirmPassword && (
                <p className="text-sm text-destructive">{errors.confirmPassword.message}</p>
              )}
              {!errors.confirmPassword && watchedConfirmPassword && watchedNewPassword &&
                watchedConfirmPassword === watchedNewPassword && (
                  <p className="text-sm text-green-600 dark:text-green-400">密码确认一致</p>
                )}
            </div>

            {/* 操作按钮 */}
            <div className="flex gap-3 pt-2">
              <LoadingButton
                type="submit"
                loading={passwordModifyMutation.isPending}
                disabled={
                  !watchedNewPassword ||
                  !watchedConfirmPassword ||
                  (hasPassword && !form.watch('oldPassword')) ||
                  watchedNewPassword !== watchedConfirmPassword ||
                  !!Object.keys(errors).length
                }
                className="w-full"
              >
                {hasPassword ? '修改密码' : '设置密码'}
              </LoadingButton>
            </div>
          </form>

          {/* 安全提示 */}
          <div className="text-xs text-muted-foreground bg-muted/30 p-3 rounded-md">
            <h4 className="font-medium mb-1">安全提示：</h4>
            <ul className="space-y-1 list-disc list-inside">
              <li>密码长度至少6位，建议包含字母、数字和特殊字符</li>
              <li>不要使用过于简单或容易被猜到的密码</li>
              <li>定期更换密码，提高账号安全性</li>
              {hasPassword && <li>修改密码后请重新登录</li>}
            </ul>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
