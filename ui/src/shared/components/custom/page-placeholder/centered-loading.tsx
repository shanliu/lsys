import { cn } from "@shared/lib/utils";
import { PageSkeletonBaseProps } from "./base";

export interface CenteredLoadingProps extends PageSkeletonBaseProps {
    /** 
     * 加载提示变体
     * - 'content': 无边框模式（仅居中加载动画）
     * - 'card': 带卡片容器的加载提示
     * - 'page': 上下两栏布局（上面过滤块-小，下面内容块-大，每块中间loading）
     * - 'layout': 左右两栏布局（左侧边栏，右侧上下两栏，每块中间loading）
     */
    variant?: 'content' | 'card' | 'page' | 'layout';
    /** 图标大小 */
    iconSize?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
}

/**
 * CenteredLoading
 * 居中加载提示组件，支持4种展示模式
 */
export function CenteredLoading({
    className,
    variant = 'content',
    iconSize = 'lg',
}: CenteredLoadingProps) {
    // 竖条尺寸映射
    const barSizeMap = {
        xs: { height: 16, width: 2, gap: 1 },
        sm: { height: 24, width: 3, gap: 2 },
        md: { height: 32, width: 4, gap: 3 },
        lg: { height: 48, width: 5, gap: 4 },
        xl: { height: 64, width: 6, gap: 5 },
    };

    const { height, width, gap } = barSizeMap[iconSize];

    const ariaLabel = '正在加载...';
    const ariaProps = {
        role: "status" as const,
        "aria-live": "polite" as const,
        "aria-busy": true,
        "aria-label": ariaLabel,
    };

    // 渲染加载内容（竖条起伏动画）
    const renderLoadingContent = () => (
        <div className="flex items-center justify-center" style={{ gap: `${gap}px` }}>
            {[0, 1, 2, 3, 4].map((index) => (
                <div
                    key={index}
                    className="bg-primary/40 rounded-sm"
                    style={{
                        width: `${width}px`,
                        height: `${height}px`,
                        animation: `wave 1.2s ease-in-out ${index * 0.1}s infinite`,
                    }}
                />
            ))}
            <span className="sr-only">{ariaLabel}</span>
        </div>
    );

    // 模式 4: 左右两栏布局（左侧边栏，右侧上下两栏）
    if (variant === 'layout') {
        return (
            <div className={cn("flex", className)} {...ariaProps}>
                {/* 左侧边栏 - 移动端隐藏 */}
                <div className="hidden md:flex w-64 shrink-0 border-r min-h-[calc(100vh-3rem)] flex-col items-center justify-center">
                    {renderLoadingContent()}
                </div>

                {/* 右侧内容区域（上下两栏） */}
                <div className="flex-1 sm:p-4 sm:space-y-4 min-w-0">
                    {/* 上面过滤块（较小）- 移动端隐藏 */}
                    <div className="hidden sm:flex rounded-lg border bg-card shadow-sm h-16 sm:h-24 flex-col items-center justify-center">
                        {renderLoadingContent()}
                    </div>

                    {/* 下面内容块（较大，填充剩余空间） */}
                    <div className="rounded-lg border bg-card shadow-sm flex-1 flex flex-col items-center justify-center min-h-[calc(100vh-6rem)] sm:min-h-[calc(100vh-12rem)]">
                        {renderLoadingContent()}
                    </div>
                </div>
            </div>
        );
    }

    // 模式 3: 上下两栏布局（上面过滤块-小，下面内容块-大）
    if (variant === 'page') {
        return (
            <div className={cn("sm:space-y-4 min-h-[calc(100vh-6rem)] sm:min-h-[calc(100vh-8rem)]", className)} {...ariaProps}>
                {/* 上面过滤块（较小）- 移动端隐藏 */}
                <div className="hidden sm:flex rounded-lg border bg-card shadow-sm h-16 sm:h-24 flex-col items-center justify-center">
                    {renderLoadingContent()}
                </div>

                {/* 下面内容块（较大） */}
                <div className="rounded-lg border bg-card shadow-sm min-h-[300px] sm:min-h-[500px] flex flex-col items-center justify-center">
                    {renderLoadingContent()}
                </div>
            </div>
        );
    }

    // 模式 2: 带卡片容器的加载提示
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
                {renderLoadingContent()}
            </div>
        );
    }

    // 模式 1: 无边框模式（仅居中加载动画）
    return (
        <div
            className={cn(
                "flex flex-col items-center justify-center ",
                className
            )}
            {...ariaProps}
        >
            {renderLoadingContent()}
        </div>
    );
}
