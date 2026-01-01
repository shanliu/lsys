import { accountLoginData, accountLogout } from '@shared/apis/user/account';
import { LoadingButton } from '@apps/main/components/local/sender-config/loading-button';
import { ContentDialog } from '@shared/components/custom/dialog/content-dialog';
import { Avatar, AvatarFallback } from '@shared/components/ui/avatar';
import { Badge } from '@shared/components/ui/badge';
import { Button } from '@shared/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@shared/components/ui/card';
import { useToast } from '@shared/contexts/toast-context';
import { userStore, AuthUserItem, authUserItemStatus } from '@shared/lib/auth';
import { cn, formatServerError, formatTime, TIME_STYLE } from '@shared/lib/utils';
import { useMutation } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { CheckCircle, Clock, Plus, Trash2, User, XCircle } from 'lucide-react';

export function UserSwitchPage() {
  const navigate = useNavigate();
  const toast = useToast();
  const userData = userStore((state) => state.userData);
  const useUserId = userStore((state) => state.useUserId);
  const switchUser = userStore((state) => state.switchUser);
  const invalidatedUser = userStore((state) => state.invalidatedUser);
  const delUser = userStore((state) => state.delUser);

  // 用于验证用户信息的 mutation，禁用缓存
  const validateUserMutation = useMutation({
    mutationFn: async ({ userId, bearer }: { userId: number; bearer: string }) => {
      const result = await accountLoginData(
        {
          reload_auth: true,
          auth: true,
          user: true
        },
        {
          headers: {
            Authorization: `Bearer ${bearer}`
          }
        }
      );
      return { result, userId };
    },
    // 禁用缓存
    gcTime: 0,
    retry: false,
    onSuccess: ({ result, userId }) => {
      if (result.status && result.response) {
        // 验证成功，执行切换
        const [success] = switchUser(userId);
        if (success) {
          toast.success('用户切换成功');
          // 切换成功后返回主页或之前页面
          navigate({ to: '/user' });
        } else {
          toast.error(formatServerError(result));
        }
      } else {
        // 验证失败，标记用户为无效
        const errorMsg = result.message || '用户验证失败';
        invalidatedUser(userId, errorMsg);
        toast.error(formatServerError(result));
      }
    },
    onError: (error: any, { userId }) => {
      // 网络错误或其他异常，标记用户为无效
      const errorMsg = error?.response?.data?.message || error?.message || '网络连接异常';
      invalidatedUser(userId, errorMsg);
      toast.error(formatServerError(error));
    }
  });

  const handleSwitchUser = (userId: number) => {
    if (userId === useUserId) return;

    // 获取要切换的用户信息
    const targetUser = userData.find(user => user.userId === userId);
    if (!targetUser) {
      toast.error('用户信息不存在');
      return;
    }

    // 使用 mutation 验证用户信息
    validateUserMutation.mutate({ userId, bearer: targetUser.bearer });
  };

  const handleAddUser = () => {
    // 跳转到登录页面，添加特殊参数标识来源
    navigate({
      to: '/sign-in',
      search: { from: 'user-switch' }
    });
  };

  // 用于删除用户的 mutation
  const deleteUserMutation = useMutation({
    mutationFn: async ({ userId, userName, bearer }: { userId: number; userName: string; bearer: string }) => {
      // 调用退出登录接口
      const result = await accountLogout({
        headers: {
          Authorization: `Bearer ${bearer}`
        }
      });
      return { userId, userName, result };
    },
    gcTime: 0,
    retry: false,
    onSuccess: ({ userId, userName, result }) => {
      if (result.status) {
        const deletedUser = delUser(userId);
        if (deletedUser) {
          toast.success(`已删除用户: ${userName}`);
        } else {
          toast.error('删除用户失败');
        }
      } else {
        // 接口返回失败，仍然删除本地用户数据
        delUser(userId);
        toast.error(formatServerError(result));
      }
    },
    onError: (error: any, { userId, userName }) => {
      // 网络错误或其他异常，仍然删除本地用户数据
      delUser(userId);
      toast.error(formatServerError(error));
    }
  });

  const handleDeleteUser = (userId: number, userName: string, bearer: string) => {
    deleteUserMutation.mutate({ userId, userName, bearer });
  };

  const getStatusBadge = (user: AuthUserItem) => {
    const status = authUserItemStatus(user);

    switch (status.kind) {
      case 'Ok':
        return (
          <Badge variant="default" className={cn("bg-green-100 text-green-800")}>
            <CheckCircle className={cn("w-3 h-3 mr-1")} />
            正常
          </Badge>
        );
      case 'Invalid':
        return (
          <Badge variant="destructive" title={status.msg || '用户已失效'} className="max-w-[120px] text-white">
            <XCircle className={cn("w-3 h-3 mr-1 flex-shrink-0")} />
            <span className="truncate">已失效</span>
          </Badge>
        );
      case 'Expired':
        const expiredTimeElement = formatTime(status.expiredAt * 1000, TIME_STYLE.ABSOLUTE_ELEMENT);
        return (
          <Badge variant="secondary">
            <Clock className={cn("w-3 h-3 mr-1")} />
            已过期: {expiredTimeElement}
          </Badge>
        );
      default:
        return null;
    }
  };

  const getLoginTypeDisplay = (loginType: string) => {
    switch (loginType) {
      case 'account':
        return '账号登录';
      case 'mail':
        return '邮箱登录';
      case 'sms':
        return '短信登录';
      default:
        return loginType;
    }
  };

  return (
    <div className="container mx-auto px-4 py-8 max-w-4xl">
      <div className="mb-8">
        <h1 className="text-3xl font-bold mb-2">用户切换</h1>
        <p className="text-gray-600">选择要切换的登录用户，或添加新的登录用户</p>
      </div>

      <div className="grid gap-4">
        {/* 现有用户列表 */}
        {userData.map((user) => {
          const isCurrentUser = user.userId === useUserId;
          const status = authUserItemStatus(user);

          return (
            <Card
              key={user.userId}
              className={`transition-all duration-300 hover:border-gray-300/60 ${isCurrentUser ? 'ring-1 ring-primary/60 border-primary/40' : ''
                }`}
            >
              <CardHeader className={cn("pb-1 pt-3")}>
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <Avatar className={cn("w-7 h-7")}>
                      <AvatarFallback>
                        <User className={cn("w-3 h-3")} />
                      </AvatarFallback>
                    </Avatar>
                    <div className="flex-1">
                      <CardTitle className={cn("text-sm leading-tight")}>{user.userNikeName}</CardTitle>
                      <CardDescription className={cn("text-xs leading-tight")}>
                        ID: {user.userId} · {getLoginTypeDisplay(user.loginType)}
                      </CardDescription>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    {getStatusBadge(user)}
                    {isCurrentUser && (
                      <Badge variant="default" className={cn("text-xs px-1.5 py-0")}>使用中</Badge>
                    )}
                  </div>
                </div>
              </CardHeader>
              <CardContent className={cn("pt-0 pb-2")}>
                <div className="flex justify-between items-end">
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-1 flex-1">
                    <div>
                      <p className="text-xs text-gray-600">登录时间</p>
                      <p className="text-xs font-medium">{formatTime(user.loginTime * 1000, TIME_STYLE.RELATIVE_ELEMENT)}</p>
                    </div>
                    {user.timeOut > 0 && (
                      <div>
                        <p className="text-xs text-gray-600">过期时间</p>
                        <p className="text-xs font-medium">{formatTime(user.timeOut * 1000, TIME_STYLE.ABSOLUTE_ELEMENT)}</p>
                      </div>
                    )}
                  </div>
                  {/* 只有非当前用户才显示操作按钮 */}
                  {!isCurrentUser && (
                    <div className="ml-4 flex items-center gap-2">
                      {/* 已过期或已失效的用户显示删除按钮 */}
                      {(status.kind === 'Expired' || status.kind === 'Invalid') && (
                        <ContentDialog
                          title="确认删除用户"
                          content={`确定要删除用户 "${user.userNikeName}" 吗？此操作将退出该用户的登录。`}
                          footer={(closeDialog) => (
                            <div className="flex gap-2 justify-end">
                              <Button variant="outline" onClick={closeDialog}>取消</Button>
                              <Button onClick={() => { handleDeleteUser(user.userId, user.userNikeName, user.bearer); closeDialog(); }}>确认删除</Button>
                            </div>
                          )}
                        >
                          <Button
                            className={cn("h-9 text-xs px-4")}
                            size="default"
                            variant="outline"
                          >
                            <Trash2 className={cn("w-3 h-3 mr-1")} />
                            删除
                          </Button>
                        </ContentDialog>
                      )}
                      {/* 状态正常的显示切换按钮和退出按钮 */}
                      {status.kind === 'Ok' && (
                        <>
                          <ContentDialog
                            title="确认退出用户"
                            content={`确定要退出用户 "${user.userNikeName}" 吗？`}
                            footer={(closeDialog) => (
                              <div className="flex gap-2 justify-end">
                                <Button variant="outline" onClick={closeDialog}>取消</Button>
                                <Button onClick={() => { handleDeleteUser(user.userId, user.userNikeName, user.bearer); closeDialog(); }}>确认退出</Button>
                              </div>
                            )}
                          >
                            <Button
                              className={cn("h-9 text-xs px-4")}
                              size="default"
                              variant="outline"
                              title="退出该用户"
                            >
                              <Trash2 className={cn("w-3 h-3 mr-1")} />
                              退出
                            </Button>
                          </ContentDialog>
                          <LoadingButton
                            onClick={() => handleSwitchUser(user.userId)}
                            className={cn("h-9 text-xs px-4")}
                            size="default"
                            variant="outline"
                            loading={validateUserMutation.isPending && validateUserMutation.variables?.userId === user.userId}
                            loadingText="切换中..."
                          >
                            <User className={cn("w-3 h-3 mr-1")} />
                            使用
                          </LoadingButton>
                        </>
                      )}
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>
          );
        })}

        {/* 添加新用户按钮 */}
        <Card className={cn("border-dashed border-2 hover:border-primary/50 transition-colors cursor-pointer")} onClick={handleAddUser}>
          <CardContent className={cn("p-3")}>
            <div className="flex items-center justify-center space-x-2">
              <Plus className={cn("w-4 h-4 text-primary")} />
              <span className="text-sm font-medium text-gray-700">添加新用户</span>
            </div>
          </CardContent>
        </Card>

        {userData.length === 0 && (
          <Card>
            <CardContent className={cn("p-8")}>
              <div className="text-center">
                <User className={cn("w-16 h-16 text-gray-400 mx-auto mb-4")} />
                <h3 className="text-lg font-semibold text-gray-700 mb-2">暂无登录用户</h3>
                <p className="text-gray-500">请先登录一个用户账号</p>
              </div>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}
