import {
  accountCheckUsername,
  accountLoginData,
  AccountLoginDataParamType,
  accountSetUsername
} from '@shared/apis/user/account';
import { LoadingButton } from '@apps/main/components/local/sender-config/loading-button';
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading';
import { Badge } from '@shared/components/ui/badge';
import { Button } from '@shared/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@shared/components/ui/card';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@shared/components/ui/form';
import { Input } from '@shared/components/ui/input';
import { Label } from '@shared/components/ui/label';
import { useToast } from '@shared/contexts/toast-context';
import { userStore } from '@shared/lib/auth';
import { cn } from '@shared/lib/utils';
import { formatServerError } from '@shared/lib/utils/format-utils';
import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import React, { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { SetNameSchema, type SetNameForm } from './set-name-schema';

export const AccountSetNamePage: React.FC = () => {
  const { success, error, warning } = useToast();
  const [usernameError, setUsernameError] = useState<string>('');
  const [usernameToCheck, setUsernameToCheck] = useState<string>('');

  // 初始化表单
  const form = useForm<SetNameForm>({
    resolver: zodResolver(SetNameSchema),
    defaultValues: {
      username: ''
    }
  });

  const { watch, setValue, formState: { errors } } = form;
  const watchedUsername = watch('username');

  // 获取当前登录信息 - 只传递 name 和 auth
  const { data: loginData, isLoading, error: queryError, refetch } = useQuery({
    queryKey: ['account-login-data'],
    queryFn: async ({ signal }) => {
      const param: AccountLoginDataParamType = {
        name: true,
        auth: true
      };
      return await accountLoginData(param, { signal });
    },
    staleTime: 0, // 立即过期，不缓存
    gcTime: 0, // 不缓存
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    retry: false
  });

  // 从查询结果中提取数据
  const authData = loginData?.response?.auth_data || {
    login_data: undefined,
    empty_password: undefined
  };

  const currentUsername = authData.login_data?.username || '';
  const canModifyUsername = authData.empty_password === false;

  // 初始化用户名输入框
  useEffect(() => {
    if (currentUsername && !watchedUsername) {
      setValue('username', currentUsername);
    }
  }, [currentUsername, watchedUsername, setValue]);

  // 检查用户名可用性 - 使用 useQuery
  const usernameCheckQuery = useQuery({
    queryKey: ['check-username', usernameToCheck],
    queryFn: async ({ signal }) => {
      return await accountCheckUsername({ name: usernameToCheck }, { signal });
    },
    enabled: !!usernameToCheck && usernameToCheck !== currentUsername && usernameToCheck.length > 0,
    staleTime: 0, // 立即过期，不缓存
    gcTime: 0, // 不缓存
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    retry: false
  });

  // 保存用户名 - 使用 useMutation
  const setUsernameMutation = useMutation({
    mutationFn: async (username: string) => {
      return await accountSetUsername({ name: username });
    },
    onSuccess: async (result, username) => {
      if (result.status) {
        success('用户名修改成功');

        // 直接更新 userStore 中的 loginData
        const currentUser = userStore.getState().current();
        if (currentUser && currentUser.loginData) {
          userStore.getState().useUser({
            ...currentUser,
            loginData: {
              ...currentUser.loginData,
              username: username
            }
          });
        }

        // 重新获取登录数据
        await refetch();
      } else {
        error(formatServerError(result));
      }
    },
    onError: (err: any) => {
      error(formatServerError(err));
    }
  });

  // 防抖检查用户名可用性
  useEffect(() => {
    if (watchedUsername && watchedUsername !== currentUsername && watchedUsername.length > 0) {
      const timeoutId = setTimeout(() => {
        setUsernameToCheck(watchedUsername);
      }, 500);

      return () => clearTimeout(timeoutId);
    } else {
      setUsernameToCheck('');
      setUsernameError('');
    }
  }, [watchedUsername, currentUsername]);

  // 处理用户名检查结果
  useEffect(() => {
    if (usernameCheckQuery.data) {
      const result = usernameCheckQuery.data;
      if (result.status && result.response) {
        if (result.response.pass !== '1') {
          setUsernameError('用户名已被使用');
        } else {
          setUsernameError('');
        }
      } else {
        setUsernameError(formatServerError(result));
      }
    } else if (usernameCheckQuery.error) {
      setUsernameError(formatServerError(usernameCheckQuery.error));
    }
  }, [usernameCheckQuery.data, usernameCheckQuery.error]);

  // 表单提交处理
  const onSubmit = (data: SetNameForm) => {
    if (!data.username || data.username === currentUsername) {
      warning('用户名未发生变化');
      return;
    }

    if (usernameError || usernameCheckQuery.isLoading) {
      error('请输入可用的用户名');
      return;
    }

    setUsernameMutation.mutate(data.username);
  };

  // 判断用户名是否可用
  const isUsernameAvailable = usernameCheckQuery.data?.status &&
    usernameCheckQuery.data?.response?.pass === '1' &&
    !usernameError;

  const isCheckingUsername = usernameCheckQuery.isLoading;

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

  return (
    <div className="px-4 py-6 space-y-6">
      <Card className={cn("mx-0 md:mx-0")}>
        <CardHeader>
          <CardTitle className={cn("flex items-center gap-2")}>
            账号设置
            <Badge variant="outline" className={cn("text-xs")}>
              {canModifyUsername ? '已设置密码' : '未设置密码'}
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent className={cn("space-y-6")}>
          {/* 当前用户名显示 */}
          <div className="space-y-2">
            <Label className={cn("text-sm font-medium text-muted-foreground")}>
              当前登录账号
            </Label>
            <div className="p-3 bg-muted rounded-md border">
              <span className="font-mono text-sm">{currentUsername || '未设置'}</span>
            </div>
          </div>

          {/* 用户名修改 */}
          {canModifyUsername && (
            <>
              {/* 安全提示 */}
              <div className="p-4 bg-warning/10 border border-warning/20 rounded-md mb-4">
                <div className="flex items-start">
                  <div className="flex-shrink-0">
                    <svg className="h-5 w-5 text-warning" viewBox="0 0 20 20" fill="currentColor">
                      <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                    </svg>
                  </div>
                  <div className="ml-3 flex-1">
                    <h3 className="text-sm font-medium text-warning">
                      账号安全提醒
                    </h3>
                    <div className="mt-1 text-sm text-muted-foreground">
                      <p>修改用户名后，您需要使用新的用户名进行登录。</p>
                    </div>
                  </div>
                </div>
              </div>

              <Form {...form}>
                <form onSubmit={form.handleSubmit(onSubmit)} className={"space-y-4"}>
                  <FormField
                    control={form.control}
                    name="username"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel htmlFor="new-username" className={cn("text-sm font-medium")}>
                          新用户名
                        </FormLabel>
                        <FormControl>
                          <div className="relative">
                            <Input
                              {...field}
                              id="new-username"
                              type="text"
                              placeholder="请输入新的用户名"
                              className={cn(
                                usernameError || errors.username
                                  ? 'border-destructive focus:border-destructive'
                                  : isUsernameAvailable
                                    ? 'border-green-500 focus:border-green-500 dark:border-green-400 dark:focus:border-green-400'
                                    : ''
                              )}
                              disabled={isCheckingUsername || setUsernameMutation.isPending}
                              onChange={(e) => {
                                field.onChange(e);
                                setUsernameError(''); // 清除自定义错误
                              }}
                            />
                            {isCheckingUsername && (
                              <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
                                <div className="animate-spin rounded-full h-4 w-4 border-2 border-primary border-t-transparent"></div>
                              </div>
                            )}
                            {isUsernameAvailable && !isCheckingUsername && (
                              <div className="absolute right-3 top-1/2 transform -translate-y-1/2 text-green-500 dark:text-green-400">
                                ✓
                              </div>
                            )}
                          </div>
                        </FormControl>
                        <FormMessage />
                        {(usernameError || errors.username) && (
                          <p className="text-sm text-destructive">{usernameError || errors.username?.message}</p>
                        )}
                        {isUsernameAvailable && !usernameError && !errors.username && (
                          <p className="text-sm text-green-600 dark:text-green-400">用户名可用</p>
                        )}
                      </FormItem>
                    )}
                  />

                  {/* 操作按钮 */}
                  <div className="flex gap-3 pt-2">
                    <LoadingButton
                      type="submit"
                      loading={setUsernameMutation.isPending}
                      disabled={
                        !watchedUsername ||
                        watchedUsername === currentUsername ||
                        !isUsernameAvailable ||
                        isCheckingUsername ||
                        !!errors.username
                      }
                      className={cn("w-full")}
                    >
                      保存用户名
                    </LoadingButton>
                  </div>
                </form>
              </Form>
            </>
          )}

          {!canModifyUsername && (
            <div className="p-4 bg-primary/10 border border-primary/20 rounded-md">
              <div className="flex items-start">
                <div className="flex-shrink-0">
                  <svg className="h-5 w-5 text-primary" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                  </svg>
                </div>
                <div className="ml-3 flex-1">
                  <h3 className="text-sm font-medium text-primary">
                    账号未设置密码，需要先设置密码才能修改用户名
                  </h3>
                  <div className="mt-2 text-sm text-muted-foreground">
                    <p>为了确保账号安全，您需要先设置登录密码。</p>
                    <p className="mt-1">设置密码后，您就可以修改用户名了。</p>
                  </div>
                  <div className="mt-3">
                    <Link to="/user/account/change-password">
                      <Button size="sm" className={cn("bg-primary hover:bg-primary/90")}>
                        前往设置密码
                      </Button>
                    </Link>
                  </div>
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
};
