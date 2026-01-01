import { cn } from "@shared/lib/utils";
import { ReactNode, cloneElement, isValidElement } from "react";
import { PageSkeletonBaseProps } from "./base";

export interface CenteredMessageProps extends PageSkeletonBaseProps {
    /** 
     * 提示变体
     * - 'content': 无边框模式（仅居中提示内容）
     * - 'card': 带卡片容器的提示
     * - 'page': 上下两栏布局（上面过滤块-小，下面内容块-大，每块中间提示）
     * - 'layout': 左右两栏布局（左侧边栏，右侧上下两栏，每块中间提示）
     */
    variant?: 'content' | 'card' | 'page' | 'layout';
    /** 提示信息 */
    message: string;
    /** 图标组件（ReactNode，可以是任何图标并设置 size 等属性） */
    icon: ReactNode;
}

/**
 * CenteredMessage
 * 居中提示组件，支持4种展示模式
 */
export function CenteredMessage({
    className,
    variant = 'content',
    message,
    icon,
}: CenteredMessageProps) {
    const ariaProps = {
        role: "status" as const,
        "aria-live": "polite" as const,
        "aria-label": message,
    };

    // 渲染提示内容（图标 + 文字）- 完整版
    const renderMessageContent = () => {
        // 尝试从 icon 中获取 size，如果没有则默认为 64
        let originalIconSize = 64;
        if (isValidElement(icon) && (icon as any).props?.size) {
            originalIconSize = (icon as any).props.size;
        }

        // 图标占1/2，padding占1/2（上下左右各占1/4）
        const iconSize = Math.round(originalIconSize * (1 / 2));
        const padding = Math.round(originalIconSize * (1 / 4));

        // 克隆图标并调整尺寸
        let resizedIcon = icon;
        if (isValidElement(icon)) {
            resizedIcon = cloneElement(icon as any, { size: iconSize });
        }

        return (
            <div className="flex flex-col items-center justify-center gap-4 text-center">
                {/* 图标容器 - 淡灰色背景 */}
                <div
                    className="text-foreground/25 dark:text-foreground/35 rounded-full bg-foreground/5 dark:bg-foreground/10"
                    style={{ padding: `${padding}px` }}
                >
                    {resizedIcon}
                </div>

                {/* 文字内容 */}
                <div className="space-y-2">
                    <p className="text-sm font-medium text-muted-foreground">
                        {message}
                    </p>
                </div>
            </div>
        );
    };

    // 渲染图标内容（仅图标）- 用于上栏和左栏，固定尺寸
    const renderIconOnly = (originalSize: number) => {
        // originalSize 是原始图标尺寸，图标缩小到1/2，padding占1/2
        const iconSize = Math.round(originalSize * (1 / 2));
        const padding = Math.round(originalSize * (1 / 4));

        // 如果 icon 是 React 元素，克隆并覆盖 size 属性
        if (isValidElement(icon)) {
            const resizedIcon = cloneElement(icon as any, { size: iconSize });
            return (
                <div className="flex items-center justify-center">
                    <div
                        className="text-foreground/25 dark:text-foreground/35 rounded-full bg-foreground/5 dark:bg-foreground/10"
                        style={{ padding: `${padding}px` }}
                    >
                        {resizedIcon}
                    </div>
                </div>
            );
        }

        // 如果不是 React 元素，直接渲染
        return (
            <div className="flex items-center justify-center">
                <div
                    className="text-foreground/25 dark:text-foreground/35 rounded-full bg-foreground/5 dark:bg-foreground/10"
                    style={{ padding: `${padding}px` }}
                >
                    {icon}
                </div>
            </div>
        );
    };

    // 模式 4: 左右两栏布局（左侧边栏，右侧上下两栏）
    if (variant === 'layout') {
        return (
            <div className={cn("flex", className)} {...ariaProps}>
                {/* 左侧边栏 - 仅显示图标（固定 56px，更大）- 移动端隐藏 */}
                <div className="hidden md:flex w-64 shrink-0 border-r min-h-[calc(100vh-3rem)] flex-col items-center justify-center p-6">
                    {renderIconOnly(56)}
                </div>

                {/* 右侧内容区域（上下两栏） */}
                <div className="flex-1 sm:p-4 sm:space-y-4 min-w-0">
                    {/* 上面过滤块（较小）- 仅显示图标（固定 32px）- 移动端隐藏 */}
                    <div className="hidden sm:flex rounded-lg border bg-card shadow-sm h-16 sm:h-24 flex-col items-center justify-center p-4 sm:p-6">
                        {renderIconOnly(32)}
                    </div>

                    {/* 下面内容块（较大，填充剩余空间）- 显示图标和文字 */}
                    <div className="rounded-lg border bg-card shadow-sm flex-1 flex flex-col items-center justify-center p-4 sm:p-6 min-h-[calc(100vh-6rem)] sm:min-h-[calc(100vh-12rem)]">
                        {renderMessageContent()}
                    </div>
                </div>
            </div>
        );
    }

    // 模式 3: 上下两栏布局（上面过滤块-小，下面内容块-大）
    if (variant === 'page') {
        return (
            <div className={cn("sm:space-y-4 min-h-[calc(100vh-6rem)] sm:min-h-[calc(100vh-8rem)]", className)} {...ariaProps}>
                {/* 上面过滤块（较小）- 仅显示图标（固定 32px）- 移动端隐藏 */}
                <div className="hidden sm:flex rounded-lg border bg-card shadow-sm h-16 sm:h-24 flex-col items-center justify-center p-4 sm:p-6">
                    {renderIconOnly(32)}
                </div>

                {/* 下面内容块（较大）- 显示图标和文字 */}
                <div className="rounded-lg border bg-card shadow-sm min-h-[300px] sm:min-h-[500px] flex flex-col items-center justify-center p-4 sm:p-6">
                    {renderMessageContent()}
                </div>
            </div>
        );
    }

    // 模式 2: 带卡片容器的提示
    if (variant === 'card') {
        return (
            <div
                className={cn(
                    "flex flex-col items-center justify-center rounded-lg border bg-card text-card-foreground shadow-sm",
                    "min-h-[300px] p-8",
                    className
                )}
                {...ariaProps}
            >
                {renderMessageContent()}
            </div>
        );
    }

    // 模式 1: 无边框模式（仅居中提示内容）
    return (
        <div
            className={cn(
                "flex flex-col items-center justify-center min-h-[200px] py-8",
                className
            )}
            {...ariaProps}
        >
            {renderMessageContent()}
        </div>
    );
}
