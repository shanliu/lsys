import { getSafeRedirectPath } from "@shared/lib/auth";
import { useNavigate } from "@tanstack/react-router";

/**
 * 处理登录后的重定向逻辑
 * @returns 返回一个函数，可以调用它来执行重定向
 */
export function useAuthRedirect() {
  const navigate = useNavigate();

  return () => {
    const params = new URLSearchParams(window.location.search);
    const redirectUri = params.get("redirect_uri");
    const to = getSafeRedirectPath(redirectUri, "/user");
    if (typeof to === 'string' && /^https?:\/\//i.test(to)) {
      window.location.replace(to);
      return;
    }
    navigate({
      to: to as any,
      replace: true
    });
  };
}
