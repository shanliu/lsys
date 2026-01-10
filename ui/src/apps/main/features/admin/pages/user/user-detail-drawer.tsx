import { CenteredLoading } from "@/shared/components/custom/page-placeholder/centered-loading";
import type { TypedDictData } from "@apps/main/hooks/use-dict-data";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import {
  systemUserAccountDetail,
  systemUserLoginHistory,
  systemUserLogout,
  type AddressItemType,
  type EmailItemType,
  type MobileItemType,
  type SystemUserAccountDetailDataType,
  type SystemUserAccountDetailParamType,
  type SystemUserLoginHistoryItemType,
  type SystemUserLoginHistoryParamType,
  type SystemUserLogoutParamType,
} from "@shared/apis/admin/user";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { ScrollArea } from "@shared/components/ui/scroll-area";
import { Separator } from "@shared/components/ui/separator";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@shared/components/ui/sheet";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@shared/components/ui/tabs";
import { useIsMobile } from "@shared/hooks/use-mobile";
import {
  cn,
  formatServerError,
  formatTime,
  getQueryResponseData,
  TIME_STYLE,
} from "@shared/lib/utils";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Clock,
  History,
  LogOut,
  Mail,
  MapPin,
  Phone,
  User,
} from "lucide-react";
import { useMemo, useState } from "react";

// ==================== 类型定义 ====================

interface UserDetailDrawerProps {
  userId: number;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  dictData: TypedDictData<["admin_user"]>;
}

interface StatusMapper {
  getClass: (status: number) => string;
  getText: (status: number) => string;
}

// ==================== 主组件 ====================

export function UserDetailDrawer({
  userId,
  open,
  onOpenChange,
  dictData,
}: UserDetailDrawerProps) {
  const queryClient = useQueryClient();
  const isMobile = useIsMobile();
  const [activeTab, setActiveTab] = useState("basic");

  // 获取字典数据
  const accountStatusDict = dictData.account_status;
  const emailStatusDict = dictData.email_status;
  const mobileStatusDict = dictData.mobile_status;
  const sessionStatusDict = dictData.session_status;

  // 获取用户详情
  const userDetailParams: SystemUserAccountDetailParamType = {
    account_id: userId,
    enable: true,
    base: true,
    name: true,
    info: true,
    address: true,
    email: [1, 2],
    mobile: [1, 2],
  };

  const {
    data: userDetailData,
    isLoading: userDetailIsLoading,
    isError: userDetailIsError,
    error: userDetailError,
  } = useQuery({
    queryKey: ["systemUserAccountDetail", userDetailParams],
    queryFn: ({ signal }) =>
      systemUserAccountDetail(userDetailParams, { signal }),
    enabled: open && userId > 0,
  });

  const userDetail = getQueryResponseData<SystemUserAccountDetailDataType | null>(
    userDetailData,
    null
  );

  // 获取登录历史 - 使用 limit.forward 参数
  const loginHistoryParams: SystemUserLoginHistoryParamType = {
    user_id: userId,
    app_id: 0,
    oauth_app_id: 0,
    is_enable: null,
    count_num: false,
    limit: {
      limit: 20,
      forward: true,
    },
  };

  const {
    data: loginHistoryData,
    isLoading: loginHistoryIsLoading,
  } = useQuery({
    queryKey: ["systemUserLoginHistory", loginHistoryParams],
    queryFn: ({ signal }) =>
      systemUserLoginHistory(loginHistoryParams, { signal }),
    enabled: open && userId > 0 && activeTab === "login-history",
  });

  const loginHistory = getQueryResponseData<SystemUserLoginHistoryItemType[]>(
    loginHistoryData,
    []
  );

  // 强制登出 mutation
  const logoutMutation = useMutation({
    mutationFn: (params: SystemUserLogoutParamType) =>
      systemUserLogout(params),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["systemUserLoginHistory", loginHistoryParams],
      });
    },
  });

  // 处理强制登出
  const handleLogout = (item: SystemUserLoginHistoryItemType) => {
    logoutMutation.mutate({
      app_id: item.app_id,
      oauth_app_id: item.oauth_app_id,
      token_data: item.token_data,
    });
  };

  // 创建状态映射器
  const accountStatus = useMemo(
    () =>
      accountStatusDict
        ? createStatusMapper(
          {
            1: "neutral",
            2: "success",
          },
          (status) =>
            accountStatusDict.getLabel(String(status)) || String(status)
        )
        : null,
    [accountStatusDict]
  );

  const emailStatus = useMemo(
    () =>
      emailStatusDict
        ? createStatusMapper(
          {
            1: "warning",
            2: "success",
          },
          (status) =>
            emailStatusDict.getLabel(String(status)) || String(status)
        )
        : null,
    [emailStatusDict]
  );

  const mobileStatus = useMemo(
    () =>
      mobileStatusDict
        ? createStatusMapper(
          {
            1: "warning",
            2: "success",
          },
          (status) =>
            mobileStatusDict.getLabel(String(status)) || String(status)
        )
        : null,
    [mobileStatusDict]
  );

  const sessionStatus = useMemo(
    () =>
      sessionStatusDict
        ? createStatusMapper(
          {
            1: "success",
            2: "danger",
            3: "warning",
          },
          (status) =>
            sessionStatusDict.getLabel(String(status)) || String(status)
        )
        : null,
    [sessionStatusDict]
  );

  // 计算抽屉宽度 - 移动端全宽，PC端固定宽度
  const sheetWidth = isMobile ? "w-full" : "w-[700px] sm:max-w-[700px]";

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className={cn(sheetWidth, "p-0")}>
        <SheetHeader className="px-6 pt-6 pb-4 border-b">
          <SheetTitle className="flex items-center gap-2">
            <User className="h-5 w-5" />
            用户详情 {userId > 0 && `#${userId}`}
          </SheetTitle>
        </SheetHeader>

        <Tabs
          value={activeTab}
          onValueChange={setActiveTab}
          className="flex flex-col h-[calc(100%-80px)]"
        >
          <TabsList
            className={cn(
              "w-full justify-start rounded-none border-b px-6 h-12 bg-transparent",
              isMobile && "overflow-x-auto flex-nowrap"
            )}
          >
            <TabsTrigger value="basic" className="data-[state=active]:bg-muted flex-shrink-0">
              基本信息
            </TabsTrigger>
            <TabsTrigger value="contact" className="data-[state=active]:bg-muted flex-shrink-0">
              联系方式
            </TabsTrigger>
            <TabsTrigger value="address" className="data-[state=active]:bg-muted flex-shrink-0">
              地址信息
            </TabsTrigger>
            <TabsTrigger value="login-history" className="data-[state=active]:bg-muted flex-shrink-0">
              登录历史
            </TabsTrigger>
          </TabsList>

          <ScrollArea className="flex-1">
            <div className={cn("p-6", isMobile && "p-4")}>
              {userDetailIsLoading && (
                 <CenteredLoading variant="content" />
              )}

              {userDetailIsError && (
                <div className="text-sm text-destructive text-center py-8">
                  {formatServerError(userDetailError, "加载用户详情失败")}
                </div>
              )}

              {userDetail && (
                <>
                  {/* 基本信息 Tab */}
                  <TabsContent value="basic" className="mt-0 space-y-6">
                    <BasicInfoSection
                      userDetail={userDetail}
                      accountStatus={accountStatus}
                      isMobile={isMobile}
                    />
                  </TabsContent>

                  {/* 联系方式 Tab */}
                  <TabsContent value="contact" className="mt-0 space-y-6">
                    <ContactInfoSection
                      emails={userDetail.email || []}
                      mobiles={userDetail.mobile || []}
                      emailStatus={emailStatus}
                      mobileStatus={mobileStatus}
                      isMobile={isMobile}
                    />
                  </TabsContent>

                  {/* 地址信息 Tab */}
                  <TabsContent value="address" className="mt-0 space-y-6">
                    <AddressInfoSection
                      addresses={userDetail.address || []}
                      isMobile={isMobile}
                    />
                  </TabsContent>

                  {/* 登录历史 Tab */}
                  <TabsContent value="login-history" className="mt-0 space-y-4">
                    <LoginHistorySection
                      loginHistory={loginHistory}
                      isLoading={loginHistoryIsLoading}
                      sessionStatus={sessionStatus}
                      onLogout={handleLogout}
                      isLoggingOut={logoutMutation.isPending}
                      isMobile={isMobile}
                    />
                  </TabsContent>
                </>
              )}
            </div>
          </ScrollArea>
        </Tabs>
      </SheetContent>
    </Sheet>
  );
}

// ==================== 详情子组件 ====================

// 基本信息组件
function BasicInfoSection({
  userDetail,
  accountStatus,
  isMobile,
}: {
  userDetail: SystemUserAccountDetailDataType;
  accountStatus: StatusMapper | null;
  isMobile: boolean;
}) {
  return (
    <div className="space-y-6">
      {/* 账号信息 */}
      <div className="space-y-4">
        <h3 className="text-lg font-medium flex items-center gap-2">
          <User className="h-4 w-4" />
          账号信息
        </h3>
        <div className={cn(
          "gap-4 bg-muted/30 rounded-lg p-4",
          isMobile ? "grid grid-cols-1" : "grid grid-cols-2"
        )}>
          <InfoItem label="用户ID" value={String(userDetail.user.id)} copyable />
          <InfoItem label="昵称" value={userDetail.user.nickname || "-"} />
          <InfoItem label="用户名" value={userDetail.name?.username || "-"} copyable />
          <div>
            <label className="text-sm font-medium text-muted-foreground">状态</label>
            <div className="mt-1">
              {accountStatus ? (
                <Badge className={accountStatus.getClass(userDetail.user.status)}>
                  {accountStatus.getText(userDetail.user.status)}
                </Badge>
              ) : (
                <Badge variant="secondary">
                  {userDetail.user.status === 2 ? "启用" : "初始"}
                </Badge>
              )}
            </div>
          </div>
          <InfoItem
            label="注册时间"
            value={formatTime(userDetail.user.add_time, TIME_STYLE.ABSOLUTE_TEXT) as string}
          />
          <InfoItem
            label="确认时间"
            value={
              userDetail.user.confirm_time
                ? (formatTime(userDetail.user.confirm_time, TIME_STYLE.ABSOLUTE_TEXT) as string)
                : "-"
            }
          />
          <InfoItem
            label="最后修改"
            value={formatTime(userDetail.user.change_time, TIME_STYLE.ABSOLUTE_TEXT) as string}
          />
          <InfoItem
            label="是否设置密码"
            value={userDetail.user.password_id > 0 ? "是" : "否"}
          />
        </div>
      </div>

      {/* 统计信息 */}
      <div className="space-y-4">
        <h3 className="text-lg font-medium">统计信息</h3>
        <div className={cn(
          "gap-4",
          isMobile ? "grid grid-cols-2" : "grid grid-cols-4"
        )}>
          <StatCard label="邮箱数量" value={userDetail.user.email_count} />
          <StatCard label="手机数量" value={userDetail.user.mobile_count} />
          <StatCard label="地址数量" value={userDetail.user.address_count} />
          <StatCard label="外部账号" value={userDetail.user.external_count} />
        </div>
      </div>

      {/* 个人信息 */}
      {userDetail.info && (
        <>
          <Separator />
          <div className="space-y-4">
            <h3 className="text-lg font-medium">个人信息</h3>
            <div className={cn(
              "gap-4 bg-muted/30 rounded-lg p-4",
              isMobile ? "grid grid-cols-1" : "grid grid-cols-2"
            )}>
              <InfoItem
                label="性别"
                value={
                  userDetail.info.gender === "1"
                    ? "男"
                    : userDetail.info.gender === "2"
                      ? "女"
                      : "未设置"
                }
              />
              <InfoItem label="生日" value={userDetail.info.birthday || "-"} />
              <InfoItem label="注册来源" value={userDetail.info.reg_from || "-"} />
              <InfoItem label="注册IP" value={userDetail.info.reg_ip || "-"} copyable />
              <InfoItem label="头像" value={userDetail.info.headimg || "-"} />
            </div>
          </div>
        </>
      )}
    </div>
  );
}

// 联系方式组件
function ContactInfoSection({
  emails,
  mobiles,
  emailStatus,
  mobileStatus,
  isMobile,
}: {
  emails: EmailItemType[];
  mobiles: MobileItemType[];
  emailStatus: StatusMapper | null;
  mobileStatus: StatusMapper | null;
  isMobile: boolean;
}) {
  return (
    <div className="space-y-6">
      {/* 邮箱信息 */}
      <div className="space-y-4">
        <h3 className="text-lg font-medium flex items-center gap-2">
          <Mail className="h-4 w-4" />
          邮箱信息 ({emails.length})
        </h3>
        {emails.length > 0 ? (
          <div className="space-y-3">
            {emails.map((email) => (
              <div key={email.id} className="p-4 bg-muted/30 rounded-lg border">
                <div className={cn(
                  "flex gap-2",
                  isMobile ? "flex-col items-start" : "items-center justify-between"
                )}>
                  <CopyableText value={email.email} className="text-sm font-medium" />
                  {emailStatus ? (
                    <Badge className={emailStatus.getClass(email.status)}>
                      {emailStatus.getText(email.status)}
                    </Badge>
                  ) : (
                    <Badge variant={email.status === 2 ? "default" : "secondary"}>
                      {email.status === 2 ? "已验证" : "待验证"}
                    </Badge>
                  )}
                </div>
                <div className={cn(
                  "mt-2 gap-2 text-xs text-muted-foreground",
                  isMobile ? "flex flex-col" : "grid grid-cols-2"
                )}>
                  <span>修改时间: {formatTime(email.change_time, TIME_STYLE.ABSOLUTE_TEXT)}</span>
                  <span>
                    确认时间:{" "}
                    {email.confirm_time && Number(email.confirm_time) > 0
                      ? formatTime(email.confirm_time, TIME_STYLE.ABSOLUTE_TEXT)
                      : "未确认"}
                  </span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-sm text-muted-foreground text-center py-4">暂无邮箱信息</div>
        )}
      </div>

      <Separator />

      {/* 手机信息 */}
      <div className="space-y-4">
        <h3 className="text-lg font-medium flex items-center gap-2">
          <Phone className="h-4 w-4" />
          手机信息 ({mobiles.length})
        </h3>
        {mobiles.length > 0 ? (
          <div className="space-y-3">
            {mobiles.map((mobile) => (
              <div key={mobile.id} className="p-4 bg-muted/30 rounded-lg border">
                <div className={cn(
                  "flex gap-2",
                  isMobile ? "flex-col items-start" : "items-center justify-between"
                )}>
                  <CopyableText
                    value={`+${mobile.area_code} ${mobile.mobile}`}
                    className="text-sm font-medium"
                  />
                  {mobileStatus ? (
                    <Badge className={mobileStatus.getClass(mobile.status)}>
                      {mobileStatus.getText(mobile.status)}
                    </Badge>
                  ) : (
                    <Badge variant={mobile.status === 2 ? "default" : "secondary"}>
                      {mobile.status === 2 ? "已验证" : "待验证"}
                    </Badge>
                  )}
                </div>
                <div className={cn(
                  "mt-2 gap-2 text-xs text-muted-foreground",
                  isMobile ? "flex flex-col" : "grid grid-cols-2"
                )}>
                  <span>修改时间: {formatTime(mobile.change_time, TIME_STYLE.ABSOLUTE_TEXT)}</span>
                  <span>
                    确认时间:{" "}
                    {mobile.confirm_time && Number(mobile.confirm_time) > 0
                      ? formatTime(mobile.confirm_time, TIME_STYLE.ABSOLUTE_TEXT)
                      : "未确认"}
                  </span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-sm text-muted-foreground text-center py-4">暂无手机信息</div>
        )}
      </div>
    </div>
  );
}

// 地址信息组件
function AddressInfoSection({
  addresses,
  isMobile,
}: {
  addresses: AddressItemType[];
  isMobile: boolean;
}) {
  return (
    <div className="space-y-4">
      <h3 className="text-lg font-medium flex items-center gap-2">
        <MapPin className="h-4 w-4" />
        地址信息 ({addresses.length})
      </h3>
      {addresses.length > 0 ? (
        <div className="space-y-3">
          {addresses.map((address) => (
            <div key={address.id} className="p-4 bg-muted/30 rounded-lg border">
              <div className={cn(
                "flex gap-2 mb-2",
                isMobile ? "flex-col items-start" : "items-center justify-between"
              )}>
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="font-medium">{address.name}</span>
                  <CopyableText value={address.mobile} className="text-sm text-muted-foreground" />
                </div>
                <Badge variant={address.status === 1 ? "default" : "secondary"}>
                  {address.status === 1 ? "正常" : "停用"}
                </Badge>
              </div>
              <div className="text-sm text-muted-foreground">
                <p>{address.country_code} | {address.address_info}</p>
                <p className="mt-1">{address.address_detail}</p>
              </div>
              <div className="mt-2 text-xs text-muted-foreground">
                修改时间: {formatTime(address.change_time, TIME_STYLE.ABSOLUTE_TEXT)}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="text-sm text-muted-foreground text-center py-4">暂无地址信息</div>
      )}
    </div>
  );
}

// 登录历史组件
function LoginHistorySection({
  loginHistory,
  isLoading,
  sessionStatus,
  onLogout,
  isLoggingOut,
  isMobile,
}: {
  loginHistory: SystemUserLoginHistoryItemType[];
  isLoading: boolean;
  sessionStatus: StatusMapper | null;
  onLogout: (item: SystemUserLoginHistoryItemType) => void;
  isLoggingOut: boolean;
  isMobile: boolean;
}) {
  if (isLoading) {
    return <CenteredLoading variant="content" />;
  }

  return (
    <div className="space-y-4">
      <h3 className="text-lg font-medium flex items-center gap-2">
        <History className="h-4 w-4" />
        登录历史 ({loginHistory.length})
      </h3>
      {loginHistory.length > 0 ? (
        <div className="space-y-3">
          {loginHistory.map((item) => (
            <div key={item.id} className="p-4 bg-muted/30 rounded-lg border">
              <div className={cn(
                "flex gap-2 mb-2",
                isMobile ? "flex-col items-start" : "items-center justify-between"
              )}>
                <div className="flex items-center gap-2">
                  <Clock className="h-4 w-4 text-muted-foreground" />
                  <span className="text-sm">
                    {formatTime(item.add_time, TIME_STYLE.ABSOLUTE_TEXT)}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  {sessionStatus ? (
                    <Badge className={sessionStatus.getClass(item.status)}>
                      {sessionStatus.getText(item.status)}
                    </Badge>
                  ) : (
                    <Badge variant={item.status === 1 ? "default" : "secondary"}>
                      {item.status === 1 ? "正常" : "已退出"}
                    </Badge>
                  )}
                  {item.status === 1 && (
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={() => onLogout(item)}
                      disabled={isLoggingOut}
                      className={cn(isMobile && "text-xs px-2 py-1 h-7")}
                    >
                      <LogOut className="h-3 w-3 mr-1" />
                      强制退出
                    </Button>
                  )}
                </div>
              </div>
              <div className={cn(
                "gap-2 text-xs text-muted-foreground",
                isMobile ? "flex flex-col" : "grid grid-cols-2"
              )}>
                <span>设备: {item.device_name || item.device_id || "-"}</span>
                <span>
                  IP: <CopyableText value={item.login_ip} className="inline" />
                </span>
                <span>登录类型: {item.login_type || "-"}</span>
                <span>
                  过期时间:{" "}
                  {item.expire_time
                    ? formatTime(item.expire_time, TIME_STYLE.ABSOLUTE_TEXT)
                    : "-"}
                </span>
                <span>应用ID: {item.app_id}</span>
                <span>OAuth应用ID: {item.oauth_app_id}</span>
              </div>
              {item.token_data && (
                <div className="mt-2">
                  <CopyableText
                    value={item.token_data}
                    className="text-xs font-mono text-muted-foreground break-all"
                  />
                </div>
              )}
            </div>
          ))}
        </div>
      ) : (
        <div className="text-sm text-muted-foreground text-center py-4">暂无登录历史</div>
      )}
    </div>
  );
}

// 信息项组件
function InfoItem({
  label,
  value,
  copyable = false,
}: {
  label: string;
  value: string;
  copyable?: boolean;
}) {
  return (
    <div>
      <label className="text-sm font-medium text-muted-foreground">{label}</label>
      <div className="mt-1 text-sm">
        {copyable && value !== "-" ? <CopyableText value={value} /> : <span>{value}</span>}
      </div>
    </div>
  );
}

// 统计卡片组件
function StatCard({ label, value }: { label: string; value: number }) {
  return (
    <div className="bg-muted/30 rounded-lg p-3 text-center border">
      <div className="text-2xl font-bold">{value}</div>
      <div className="text-xs text-muted-foreground">{label}</div>
    </div>
  );
}
