import { userLogout } from "@shared/apis/user/account";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@shared/components/ui/popover";
import { Separator } from "@shared/components/ui/separator";
import { Sheet, SheetContent, SheetTrigger } from "@shared/components/ui/sheet";
import { useToast } from "@shared/contexts/toast-context";
import { userStore } from "@shared/lib/auth";
import { cn, formatServerError, formatTime, TIME_STYLE } from "@shared/lib/utils";
import { useMutation } from "@tanstack/react-query";
import { Link, useLocation, useNavigate } from "@tanstack/react-router";
import { Building2, Calendar, Clock, LogOut, Shield, Smartphone, User } from "lucide-react";
import { useState } from "react";

// 判断是否为中文字符
function isChinese(char: string): boolean {
  return /[\u4e00-\u9fa5]/.test(char);
}

// 获取头像显示文本
function getAvatarText(nickname: string | undefined): string {
  if (!nickname || nickname.length === 0) return "?";

  // 如果第一个字符是中文，只显示第一个字符
  if (isChinese(nickname[0])) {
    return nickname[0].toUpperCase();
  }

  // 如果前两个字符都不是中文，显示前两个字符
  if (nickname.length >= 2 && !isChinese(nickname[0]) && !isChinese(nickname[1])) {
    return nickname.substring(0, 2).toUpperCase();
  }

  // 其他情况显示第一个字符
  return nickname[0].toUpperCase();
}

// 头像组件
function UserAvatar({ nickname, size = 22, fontSize = 11 }: { nickname: string | undefined; size?: number; fontSize?: number }) {
  const avatarText = getAvatarText(nickname);

  return (
    <div
      className={cn(
        "rounded-full flex items-center justify-center font-semibold shadow-sm",
        "bg-muted text-muted-foreground"
      )}
      style={{
        width: `${size}px`,
        height: `${size}px`,
        fontSize: `${fontSize}px`
      }}
    >
      {avatarText}
    </div>
  );
}

export function MainNavUserInfo() {
  const userData = userStore((state) => state.current)();
  const navigate = useNavigate();
  const location = useLocation();
  const toast = useToast();
  const [open, setOpen] = useState(false);

  const mutation = useMutation({
    mutationFn: async () => {
      const res = await userLogout();
      if (res.status) {
        toast.success("退出成功");
        navigate({ to: "/sign-in", replace: true });
      }
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    }
  });

  // 在切换用户页面不显示用户信息
  if (location.pathname === "/switch-user") {
    return null;
  }

  return (
    <div className="flex items-center gap-1">


      {/* User Info */}
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          {userData ? (
            <Button
              variant="ghost"
              size="icon"
              className={cn("h-9 w-9 rounded-full p-0")}
            >
              <UserAvatar nickname={userData.userNikeName} />
            </Button>
          ) : (
            <Link to="/sign-in" className={cn("text-sm text-primary")} activeProps={{ 'data-active': true }}>
              <Button size="sm" variant="outline" className={cn("w-full")}>
                登录
              </Button>
            </Link>
          )}
        </PopoverTrigger>
        {userData && (
          <PopoverContent
            className={cn(
              "w-96 p-0 overflow-hidden",
              "bg-popover text-popover-foreground",
              "rounded-lg border shadow-xl",
              "animate-in data-[side=bottom]:slide-in-from-top-2",
              "data-[side=top]:slide-in-from-bottom-2",
              "data-[side=right]:slide-in-from-left-2",
              "data-[side=left]:slide-in-from-right-2",
            )}
            align="end"
            sideOffset={8}
          >

            {/* 用户头像和基本信息 */}
            <div className="bg-gradient-to-r from-primary/5 to-primary/10 p-6">
              <div className="flex items-center gap-4">
                <UserAvatar nickname={userData.userNikeName} size={64} fontSize={24} />
                <div className="flex-1 min-w-0">
                  <div className="font-semibold text-lg text-foreground truncate">{userData.userNikeName}</div>
                  <div className="flex items-center gap-2 mt-1">
                    <User className={cn("w-4 h-4 text-muted-foreground")} />
                    <span className="text-sm text-muted-foreground">用户ID: {userData.userId}</span>
                  </div>
                  <div className="flex items-center gap-2 mt-1">
                    <Shield className={cn("w-4 h-4 text-muted-foreground")} />
                    <Badge variant="secondary" className={cn("text-xs px-2 py-1")}>{userData.loginType}</Badge>
                  </div>
                </div>
              </div>
            </div>

            {/* 应用信息 */}
            {userData.appData && (
              <div className="px-6 py-4 bg-muted/30">
                <div className="flex items-center gap-2 mb-3">
                  <Building2 className={cn("w-5 h-5 text-primary")} />
                  <span className="text-sm font-semibold text-foreground">当前应用</span>
                </div>
                <div className="bg-background rounded-lg p-4 border border-border/50 space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="text-base font-medium text-foreground">{userData.appData.appName}</span>
                    <Badge variant="outline" className={cn("text-xs font-mono bg-muted/50")}>
                      {userData.appData.clientId}
                    </Badge>
                  </div>
                  <div className="grid grid-cols-2 gap-3 text-xs">
                    <div className="space-y-1">
                      <span className="text-muted-foreground">应用ID</span>
                      <div className="font-medium text-foreground">{userData.appData.appId}</div>
                    </div>
                    <div className="space-y-1">
                      <span className="text-muted-foreground">切换时间</span>
                      <div className="font-medium text-foreground">
                        {formatTime(new Date(userData.appData.changeTime * 1000), TIME_STYLE.ABSOLUTE_ELEMENT)}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* 登录详细信息 */}
            <div className="px-6 py-3">
              <div className="space-y-2 text-xs">
                <div className="flex items-center gap-2">
                  <Clock className={cn("w-3 h-3 text-muted-foreground")} />
                  <span className="text-muted-foreground">登录时间:</span>
                  <span className="text-muted-foreground font-mono text-right ml-auto">
                    {formatTime(new Date(userData.loginTime * 1000), TIME_STYLE.ABSOLUTE_ELEMENT)}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <Calendar className={cn("w-3 h-3 text-muted-foreground")} />
                  <span className="text-muted-foreground">过期时间:</span>
                  <span className="text-muted-foreground font-mono text-right ml-auto">
                    {formatTime(new Date(userData.timeOut * 1000), TIME_STYLE.ABSOLUTE_ELEMENT)}
                  </span>
                </div>
              </div>
            </div>

            <Separator />

            {/* 操作按钮 */}
            <div className="p-6 pt-4 bg-muted/20">
              <div className="grid grid-cols-2 gap-3">
                <Link to="/switch-user" className={cn("flex-1")} onClick={() => setOpen(false)}>
                  <Button variant="outline" size="sm" className={cn("w-full h-9")}>
                    <Smartphone className={cn("w-4 h-4 mr-2")} />
                    切换用户
                  </Button>
                </Link>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={mutation.isPending}
                  onClick={() => {
                    mutation.mutate();
                  }}
                  className={cn("flex-1 h-9  hover:bg-destructive/10")}
                >
                  {mutation.isPending ? "退出中..." : "退出登录"}
                </Button>
              </div>
            </div>
          </PopoverContent>
        )}
      </Popover>
    </div>
  )
}

// 移动端用户信息组件
export function MobileUserInfo() {
  const userData = userStore((state) => state.current)();
  const toast = useToast();
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);

  const mutation = useMutation({
    mutationFn: async () => {
      const res = await userLogout();
      if (res.status) {
        toast.success("退出成功");
        navigate({ to: "/sign-in", replace: true });
        setOpen(false);
      }
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    }
  });

  return (
    <div className="flex-shrink-0 mr-2">
      <Sheet open={open} onOpenChange={setOpen}>
        <SheetTrigger asChild>
          <Button variant="ghost" size="icon" className={cn("h-9 w-9 rounded-full p-0")}>
            <UserAvatar nickname={userData?.userNikeName} />
          </Button>
        </SheetTrigger>
        <SheetContent side="right" className={cn("w-[320px] sm:w-[360px] p-0")}>
          <div className="flex flex-col h-full">
            {/* 用户头像和基本信息 */}
            <div className="bg-gradient-to-r from-primary/5 to-primary/10 p-6 pt-12">
              <div className="flex flex-col items-center gap-4">
                <UserAvatar nickname={userData?.userNikeName} size={64} fontSize={24} />
                <div className="text-center w-full">
                  <div className="font-semibold text-lg text-foreground truncate">{userData?.userNikeName || "未登录"}</div>
                  <div className="flex items-center justify-center gap-2 mt-2">
                    <User className={cn("w-4 h-4 text-muted-foreground")} />
                    <span className="text-sm text-muted-foreground">用户ID: {userData?.userId}</span>
                  </div>
                  <div className="flex items-center justify-center gap-2 mt-1">
                    <Shield className={cn("w-4 h-4 text-muted-foreground")} />
                    <Badge variant="secondary" className={cn("text-xs px-2 py-1")}>{userData?.loginType}</Badge>
                  </div>
                </div>
              </div>
            </div>

            {/* 应用信息 */}
            {userData?.appData && (
              <div className="px-4 py-4 bg-muted/30">
                <div className="flex items-center gap-2 mb-3">
                  <Building2 className={cn("w-5 h-5 text-primary")} />
                  <span className="text-sm font-semibold text-foreground">当前应用</span>
                </div>
                <div className="bg-background rounded-lg p-4 border border-border/50 space-y-3">
                  <div className="flex flex-col gap-2">
                    <span className="text-base font-medium text-foreground truncate">{userData.appData.appName}</span>
                    <Badge variant="outline" className={cn("text-xs font-mono bg-muted/50 w-fit")}>
                      {userData.appData.clientId}
                    </Badge>
                  </div>
                  <div className="grid grid-cols-1 gap-3 text-xs">
                    <div className="space-y-1">
                      <span className="text-muted-foreground">应用ID</span>
                      <div className="font-medium text-foreground">{userData.appData.appId}</div>
                    </div>
                    <div className="space-y-1">
                      <span className="text-muted-foreground">切换时间</span>
                      <div className="font-medium text-foreground">
                        {formatTime(new Date(userData.appData.changeTime * 1000), TIME_STYLE.ABSOLUTE_ELEMENT)}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* 登录详细信息 */}
            {userData && (
              <div className="px-4 py-3">
                <div className="space-y-2 text-xs">
                  <div className="flex items-center gap-2">
                    <Clock className={cn("w-3 h-3 text-muted-foreground")} />
                    <span className="text-muted-foreground">登录时间:</span>
                    <span className="text-muted-foreground font-mono text-right ml-auto text-[10px]">
                      {formatTime(new Date(userData.loginTime * 1000), TIME_STYLE.ABSOLUTE_ELEMENT)}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Calendar className={cn("w-3 h-3 text-muted-foreground")} />
                    <span className="text-muted-foreground">过期时间:</span>
                    <span className="text-muted-foreground font-mono text-right ml-auto text-[10px]">
                      {formatTime(new Date(userData.timeOut * 1000), TIME_STYLE.ABSOLUTE_ELEMENT)}
                    </span>
                  </div>
                </div>
              </div>
            )}

            <Separator />

            {/* 操作按钮 */}
            <div className="p-4 pt-4 bg-muted/20 mt-auto">
              <div className="grid grid-cols-1 gap-3">
                <Link to="/switch-user" className={cn("w-full")} onClick={() => setOpen(false)}>
                  <Button variant="outline" size="sm" className={cn("w-full h-9")}>
                    <Smartphone className={cn("w-4 h-4 mr-2")} />
                    切换用户
                  </Button>
                </Link>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={mutation.isPending}
                  onClick={() => {
                    mutation.mutate();
                  }}
                  className={cn("w-full h-9  hover:bg-destructive/10")}
                >
                  <LogOut className="h-4 w-4 mr-2" />
                  {mutation.isPending ? "退出中..." : "退出登录"}
                </Button>
              </div>
            </div>
          </div>
        </SheetContent>
      </Sheet>
    </div>
  );
}
