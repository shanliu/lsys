import {
  accountLoginData,
  AccountLoginDataParamType,
  accountSetInfo
} from '@shared/apis/user/account';
import { LoadingButton } from '@apps/main/components/local/sender-config/loading-button';
import { DatePicker } from '@shared/components/custom/input/date-picker';
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading';
import { Card, CardContent, CardHeader, CardTitle } from '@shared/components/ui/card';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@shared/components/ui/form';
import { Input } from '@shared/components/ui/input';
import { Label } from '@shared/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shared/components/ui/select';
import { useToast } from '@shared/contexts/toast-context';
import { userStore } from '@shared/lib/auth';
import { cn } from '@shared/lib/utils';
import { formatServerError } from '@shared/lib/utils/format-utils';
import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation, useQuery } from '@tanstack/react-query';
import { format } from 'date-fns';
import React, { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { SetInfoSchema, type SetInfoType } from './set-info-schema';

export const AccountSetInfoPage: React.FC = () => {
  const { success, error } = useToast();

  // 初始化表单
  const form = useForm<SetInfoType>({
    resolver: zodResolver(SetInfoSchema),
    defaultValues: {
      nikename: '',
      gender: undefined,
      birthday: ''
    }
  });

  const { formState: { errors } } = form;

  // 获取当前用户信息
  const { data: loginData, isLoading, error: queryError, refetch } = useQuery({
    queryKey: ['account-login-data-info'],
    queryFn: async ({ signal }) => {
      const param: AccountLoginDataParamType = {
        user: true,
        info: true
      };
      return await accountLoginData(param, { signal });
    },
    staleTime: 0,
    gcTime: 0,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    retry: false
  });

  // 从查询结果中提取数据
  const userAccount = loginData?.response?.user_data?.account || {};
  const userInfo = loginData?.response?.user_data?.info || {};

  const currentNikename = userAccount.nickname || '';
  const currentGender = userInfo.gender;
  const currentHeadimg = userInfo.headimg || '';
  const currentBirthday = userInfo.birthday || '';

  // 初始化表单值
  useEffect(() => {
    if (loginData) {
      form.reset({
        nikename: currentNikename,
        gender: currentGender,
        birthday: currentBirthday
      });
    }
  }, [loginData, currentNikename, currentGender, currentBirthday, form]);

  // 保存用户信息 - 使用 useMutation
  const setInfoMutation = useMutation({
    mutationFn: async (data: SetInfoType) => {
      return await accountSetInfo(data);
    },
    onSuccess: async (result, variables) => {
      if (result.status) {
        success('用户信息修改成功');

        // 如果修改了昵称，直接更新 userStore 中的昵称
        if (variables.nikename) {
          const currentUser = userStore.getState().current();
          if (currentUser) {
            userStore.getState().useUser({
              ...currentUser,
              userNikeName: variables.nikename
            });
          }
        }

        // 刷新页面显示的数据
        await refetch();
      } else {
        error(formatServerError(result));
      }
    },
    onError: (err: any) => {
      error(formatServerError(err));
    }
  });

  // 表单提交处理
  const onSubmit = (data: SetInfoType) => {
    setInfoMutation.mutate(data);
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

  return (
    <div className="px-4 py-6 space-y-6">
      <Card className={cn("mx-0 md:mx-0")}>
        <CardHeader>
          <CardTitle className={cn("flex items-center gap-2")}>
            个人信息设置
          </CardTitle>
        </CardHeader>
        <CardContent className={cn("space-y-6")}>
          {/* 头像 - moved to top */}
          <div className="space-y-2 pb-2 flex flex-col items-center">
            <Label className={cn("text-sm font-medium text-muted-foreground")}>
              头像
            </Label>
            <div className="flex items-center gap-4">
              {currentHeadimg ? (
                <img
                  src={currentHeadimg}
                  alt="用户头像"
                  className="w-20 h-20 rounded-full object-cover border"
                  onError={(e) => {
                    e.currentTarget.src = 'data:image/svg+xml,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22 width=%22100%22 height=%22100%22%3E%3Crect width=%22100%22 height=%22100%22 fill=%22%23ddd%22/%3E%3Ctext x=%2250%25%22 y=%2250%25%22 dominant-baseline=%22middle%22 text-anchor=%22middle%22 font-size=%2224%22 fill=%22%23999%22%3E头像%3C/text%3E%3C/svg%3E';
                  }}
                />
              ) : (
                <div className="w-20 h-20 rounded-full bg-muted flex items-center justify-center border">
                  <span className="text-muted-foreground text-sm">无头像</span>
                </div>
              )}
            </div>
          </div>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
              {/* 昵称 */}
              <FormField
                control={form.control}
                name="nikename"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel htmlFor="nikename" className={cn("text-sm font-medium")}>
                      昵称
                    </FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        id="nikename"
                        type="text"
                        placeholder="请输入昵称"
                        className={cn(errors.nikename ? 'border-destructive focus:border-destructive' : '')}
                        disabled={setInfoMutation.isPending}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              {/* 性别 */}
              <FormField
                control={form.control}
                name="gender"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel htmlFor="gender" className={cn("text-sm font-medium")}>
                      性别
                    </FormLabel>
                    <FormControl>
                      <Select
                        value={field.value?.toString()}
                        onValueChange={(value: string) => field.onChange(Number(value))}
                        disabled={setInfoMutation.isPending}
                      >
                        <SelectTrigger className={cn(errors.gender ? 'border-destructive' : '')}>
                          <SelectValue placeholder="请选择性别" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="0">保密</SelectItem>
                          <SelectItem value="1">男</SelectItem>
                          <SelectItem value="2">女</SelectItem>
                        </SelectContent>
                      </Select>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              {/* 生日 */}
              <FormField
                control={form.control}
                name="birthday"
                render={({ field }) => {
                  // 将日期字符串转换为本地时区的 Date 对象
                  const dateValue = field.value ? (() => {
                    const [year, month, day] = field.value.split('-').map(Number);
                    return new Date(year, month - 1, day);
                  })() : undefined;

                  return (
                    <FormItem>
                      <FormLabel htmlFor="birthday" className={cn("text-sm font-medium")}>
                        生日
                      </FormLabel>
                      <FormControl>
                        <DatePicker
                          value={dateValue}
                          onChange={(date) => {
                            field.onChange(date ? format(date, 'yyyy-MM-dd') : '');
                          }}
                          placeholder="请选择生日"
                          className={cn(errors.birthday ? 'border-destructive focus:border-destructive' : '')}
                          disabled={setInfoMutation.isPending}
                          toDate={new Date()}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  );
                }}
              />

              {/* 操作按钮 */}
              <div className="flex gap-3 pt-2">
                <LoadingButton
                  type="submit"
                  loading={setInfoMutation.isPending}
                  className={cn("w-full")}
                >
                  保存信息
                </LoadingButton>
              </div>
            </form>
          </Form>
        </CardContent>
      </Card>
    </div>
  );
};
