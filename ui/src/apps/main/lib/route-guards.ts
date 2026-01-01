import { getSafeRedirectPath, userStore } from "@shared/lib/auth";
import { redirect, type ParsedLocation } from "@tanstack/react-router";

/**
 * 检查是否来自用户切换页面
 * @param location 路由位置对象
 * @returns 如果来自用户切换页面则返回 true
 */
export function isFromUserSwitch(location: ParsedLocation): boolean {
    const searchParams = new URLSearchParams(location.search);
    const from = searchParams.get("from");
    return from === "user-switch";
}

/**
 * 检查用户认证状态并在需要时重定向到登录页面
 * 包含防止重定向循环的保护
 */
export function validateAuthAndRedirect(location: ParsedLocation) {
    const currentPath = location.pathname;
    if (currentPath.startsWith('/sign-in')) {
        return;
    }
    if (!userStore.getState().isLoggedIn()) {
        throw redirect({
            to: "/sign-in",
            search: { redirect_uri: window.location.href },
            replace: true
        });
    }
}

/**
 * 判断用户是否已经登录,如果已经登录,根据 redirect_uri 跳转
 * 用于 beforeLoad
 */
export function redirectIfLoggedIn(location: ParsedLocation) {
    if (!userStore.getState().isLoggedIn()) return;

    // 如果来源是用户切换页面，不进行重定向，允许继续登录
    if (isFromUserSwitch(location)) {
        return;
    }

    // 使用 TanStack Router 的 location 对象
    const searchParams = new URLSearchParams(location.search);
    const redirectUri = searchParams.get("redirect_uri");
    const to = getSafeRedirectPath(redirectUri, "/user");
    console.log('User is already logged in, redirecting to:', to);
    if (typeof to === 'string' && /^https?:\/\//i.test(to)) {
        window.location.replace(to);
        return;
    }
    throw redirect({
        to: to as string,
        replace: true
    });
}
