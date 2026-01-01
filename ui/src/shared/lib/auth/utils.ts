import { userStore } from ".";
import type { AuthUserItem, AuthUserStatus as AuthUserStatusType } from "./types";

export const authUserStatus = {
    Ok: (): AuthUserStatusType => ({ kind: "Ok" }),
    Invalid: (time: number, msg?: string): AuthUserStatusType => ({ kind: "Invalid", time, msg }),
    Expired: (expiredAt: number): AuthUserStatusType => ({ kind: "Expired", expiredAt }),
} as const;


// 判断 AuthUserItem 是否过期
export function authUserItemisExpired(data: AuthUserItem): boolean {
    if (data.timeOut <= 0) return false;
    const currentTime = Math.floor(Date.now() / 1000);
    return currentTime > data.timeOut;
}

// 获取 AuthUserItem 的状态
export function authUserItemStatus(data: AuthUserItem): AuthUserStatusType {
    if (data.status.kind === 'Ok') {
        if (authUserItemisExpired(data)) {
            return authUserStatus.Expired(data.timeOut);
        }
    }
    return data.status;
}

/**
 * 获取安全的重定向路径，如果重定向URI无效或不安全则返回默认路径
 * @param redirectUri 重定向的URI
 * @param defaultPath 默认重定向路径
 * @returns 返回安全的重定向路径
 */
export function getSafeRedirectPath(
    redirectUri: string | null,
    defaultPath: string,
): string {
    if (!redirectUri) return defaultPath;
    try {
        const redirectUrl = new URL(redirectUri, window.location.origin);
        const dom2 = redirectUrl.hostname.split(".").length;
        if (redirectUrl.hostname === window.location.hostname || ((
            dom2 == 2
        ) && redirectUrl.hostname === 'www.' + window.location.hostname)) {
            return redirectUri;
        }
    } catch { }
    return defaultPath;
}


// 处理登录响应并保存到store的辅助函数
export function handleLoginResponse(loginResponse: any): boolean {
    const userId = Number(loginResponse?.auth_data?.user_id);
    if (userId <= 0) return false;
    userStore.getState().useUser({
        loginType: String(loginResponse?.auth_data?.login_type),
        userId: userId,
        userNikeName: String(loginResponse?.auth_data?.user_nickname),
        loginTime: Number(loginResponse?.auth_data?.login_time),
        timeOut: Number(loginResponse?.auth_data?.time_out),
        bearer: String(loginResponse?.jwt),
        loginData: loginResponse?.auth_data?.login_data,
        status: authUserStatus.Ok(),
        appData: loginResponse?.auth_data?.app_data ? {
            appId: Number(loginResponse?.auth_data?.app_data?.app_id),
            appName: String(loginResponse?.auth_data?.app_data?.app_name),
            clientId: String(loginResponse?.auth_data?.app_data?.client_id),
            changeTime: Number(loginResponse?.auth_data?.app_data?.change_time),
        } : null,
    });
    return true;
}
