import logoImage from "@shared/assets/logo.png";
import logoImageDark from "@shared/assets/logo_black.png";
import { useTheme } from "@shared/contexts/theme-context";
import { cn, getHomeUrl } from "@shared/lib/utils";
import React from "react";

export interface AppLogoProps extends Omit<React.ImgHTMLAttributes<HTMLImageElement>, "src"> {
    /** 是否将 Logo 包装为链接到首页 */
    linkToHome?: boolean;
    /** 链接的额外 className */
    linkClassName?: string;
    /** 链接的 title 属性 */
    linkTitle?: string;
}

export const AppLogo = React.forwardRef<HTMLImageElement, AppLogoProps>(
    ({ className, alt = "lsys Logo", loading = "eager", decoding = "async", linkToHome = false, linkClassName, linkTitle, ...props }, ref) => {
        const { theme } = useTheme();

        const [systemPrefersDark, setSystemPrefersDark] = React.useState(() => {
            if (typeof window === "undefined") return false;
            return window.matchMedia("(prefers-color-scheme: dark)").matches;
        });

        React.useEffect(() => {
            if (theme !== "system") return;
            if (typeof window === "undefined") return;

            const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
            const handleChange = () => setSystemPrefersDark(mediaQuery.matches);

            handleChange();
            mediaQuery.addEventListener("change", handleChange);
            return () => mediaQuery.removeEventListener("change", handleChange);
        }, [theme]);

        const isDark = theme === "dark" || (theme === "system" && systemPrefersDark);
        const src = isDark ? logoImage : logoImageDark;

        const imgElement = (
            <img
                ref={ref}
                src={src}
                alt={alt}
                loading={loading}
                decoding={decoding}
                className={cn(className)}
                {...props}
            />
        );

        if (linkToHome) {
            return (
                <a href={getHomeUrl()} className={linkClassName} title={linkTitle}>
                    {imgElement}
                </a>
            );
        }

        return imgElement;
    }
);

AppLogo.displayName = "AppLogo";
