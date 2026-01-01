import { Button } from '@shared/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@shared/components/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shared/components/ui/select';
import { userStore, authUserItemisExpired, authUserItemStatus } from '@shared/lib/auth';
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils';
import { useNavigate } from '@tanstack/react-router';
import { useEffect, useState } from 'react';

export function SessionExpiredDialog() {
  const [isOpen, setIsOpen] = useState(false);
  const [selectedUserId, setSelectedUserId] = useState<string>('');
  const [statusInfo, setStatusInfo] = useState<{ title: string; description: string }>({
    title: '登录已过期',
    description: '您的登录会话已过期，请选择切换到其他账号或重新登录。'
  });
  const navigate = useNavigate();

  useEffect(() => {
    // 检查当前用户状态的函数
    const checkCurrentUserStatus = (state = userStore.getState()) => {
      if (state.useUserId && state.useUserId !== 0) {
        const currentUser = state.userData.find((user) => user.userId === state.useUserId);
        if (currentUser) {
          const status = authUserItemStatus(currentUser);
          // 如果当前用户状态为过期或无效，弹出对话框
          if (status.kind === 'Expired' || status.kind === 'Invalid') {
            // 根据状态类型设置不同的提示信息
            if (status.kind === 'Expired') {
              const expiredTime = formatTime(status.expiredAt * 1000, TIME_STYLE.ABSOLUTE_TEXT);
              setStatusInfo({
                title: '登录已过期',
                description: `您的登录会话已于 ${expiredTime} 过期。`
              });
            } else if (status.kind === 'Invalid') {
              const invalidTime = formatTime(status.time * 1000, TIME_STYLE.ABSOLUTE_TEXT);
              const reason = status.msg || '未知原因';
              setStatusInfo({
                title: '登录已失效',
                description: `您的登录在 ${invalidTime} 已失效，原因：${reason}。`
              });
            }
            setIsOpen(true);
          }
        }
      }
    };

    // 监听 userData 和 useUserId 的变化
    const unsubscribe = userStore.subscribe(checkCurrentUserStatus);

    // 组件挂载时检查当前用户状态
    checkCurrentUserStatus();

    return unsubscribe;
  }, []);

  // 获取当前store状态
  const { userData, useUserId, switchUser, logout } = userStore();

  // 过滤掉当前用户，获取其他有效用户
  const otherUsers = userData.filter(user => {
    if (user.userId === useUserId) return false;
    // 检查是否过期
    return !authUserItemisExpired(user);
  });

  const handleSwitchUser = () => {
    if (selectedUserId) {
      const [success] = switchUser(Number(selectedUserId));
      if (success) {
        setIsOpen(false);
        setSelectedUserId('');
      }
    }
  };

  const handleGoToLogin = () => {
    // 先退出当前登录
    logout();
    setIsOpen(false);
    navigate({
      to: "/sign-in",
      search: { redirect_uri: window.location.href }
    });
  };

  return (
    <Dialog open={isOpen} onOpenChange={() => { }}>
      <DialogContent
        className={cn("sm:max-w-md")}
        onPointerDownOutside={(e) => e.preventDefault()}
        onEscapeKeyDown={(e) => e.preventDefault()}
      >
        <DialogHeader>
          <DialogTitle>{statusInfo.title}</DialogTitle>
          <DialogDescription>
            {statusInfo.description}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {otherUsers.length > 0 ? (
            <div className="space-y-3">
              <div>
                <label className="text-sm font-medium">切换到其他账号：</label>
                <Select value={selectedUserId} onValueChange={setSelectedUserId}>
                  <SelectTrigger className={cn("mt-1")}>
                    <SelectValue placeholder="选择账号" />
                  </SelectTrigger>
                  <SelectContent className="max-h-[300px]">
                    {otherUsers.map((user) => (
                      <SelectItem key={user.userId} value={user.userId.toString()}>
                        {user.userNikeName} (ID: {user.userId})
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="flex gap-2">
                <Button
                  onClick={handleSwitchUser}
                  disabled={!selectedUserId}
                  className={cn("flex-1")}
                >
                  切换账号
                </Button>
                <Button
                  variant="outline"
                  onClick={handleGoToLogin}
                  className={cn("flex-1")}
                >
                  重新登录
                </Button>
              </div>
            </div>
          ) : (
            <div className="space-y-3">
              <Button onClick={handleGoToLogin} className={cn("w-full")}>
                前往登录页面
              </Button>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
