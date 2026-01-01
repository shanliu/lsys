import { Skeleton } from "@shared/components/ui/skeleton";
import { cn } from "@shared/lib/utils";
import { PageSkeletonBaseProps } from "./base";

export interface CardSkeletonProps extends PageSkeletonBaseProps {
    /** 卡片数量（0 表示自动计算以填满屏幕） */
    itemCount?: number;
    /** 
     * 卡片骨架屏变体
     * - 'content': 无边框模式（仅卡片网格）
     * - 'card': 带分页的完整卡片列表
     * - 'page': 带顶部过滤栏+卡片列表+分页
     * - 'layout': 左侧导航栏+顶部过滤栏+卡片列表+分页（完整布局）
     */
    variant?: 'content' | 'card' | 'page' | 'layout';
    /** 左侧导航栏菜单项数量（0 表示自动计算接近满屏） */
    sidebarMenuItems?: number;
}


export function PageSkeletonCard({
    className,
    itemCount = 0,
    variant = 'content',
    sidebarMenuItems = 0,
}: CardSkeletonProps) {
    // 自动计算卡片数量
    const calculateItemCount = (): number => {
        if (itemCount > 0) return itemCount;

        const viewportHeight = window.innerHeight;
        const viewportWidth = window.innerWidth;

        // 卡片预估高度：根据不同屏幕尺寸估算
        const cardHeight = viewportWidth >= 640 ? 280 : 320; // sm 及以上 280px，否则 320px

        // 计算列数（响应式断点）
        let columns = 1;
        if (viewportWidth >= 1280) columns = 4; // xl
        else if (viewportWidth >= 1024) columns = 3; // lg
        else if (viewportWidth >= 640) columns = 2; // sm

        // 可用高度（减去过滤栏、分页、间距等）
        let availableHeight = viewportHeight;
        if (variant === 'page' || variant === 'layout') {
            availableHeight -= 200; // 过滤栏 + 分页 + 间距
        } else if (variant === 'card') {
            availableHeight -= 100; // 分页 + 间距
        } else {
            availableHeight -= 40; // 基础间距
        }

        // 计算行数（略微超出屏幕）
        const rows = Math.ceil((availableHeight / cardHeight) * 1.2);

        return Math.max(columns * rows, 4); // 最少 4 个
    };

    // 自动计算侧边栏菜单项数量
    const calculateSidebarItems = (): number => {
        if (sidebarMenuItems > 0) return sidebarMenuItems;

        const viewportHeight = window.innerHeight;

        // 菜单项高度约 40px（包含 padding 和 margin）
        const menuItemHeight = 40;

        // 可用高度（减去标题、padding 等）
        const availableHeight = viewportHeight - 200;

        // 计算菜单项数量（接近满屏，不超出）
        const items = Math.floor(availableHeight / menuItemHeight);

        return Math.max(items, 3); // 最少 3 个
    };

    const safeItemCount = calculateItemCount();
    const safeSidebarItems = calculateSidebarItems();
    const cardPlaceholders = Array.from({ length: safeItemCount });
    const sidebarPlaceholders = Array.from({ length: safeSidebarItems });
    const filterPlaceholders = (variant === 'page' || variant === 'layout') ? Array.from({ length: 3 }) : [];
    const ariaLabel = (variant === 'page' || variant === 'layout') ? "正在加载卡片页面..." : "正在加载卡片列表...";

    const renderCardGrid = () => (
        <div className="grid gap-4 sm:gap-6 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {cardPlaceholders.map((_, index) => (
                <div key={index} className="rounded-lg border bg-card text-card-foreground shadow-sm">
                    <div className="flex flex-col space-y-1.5 p-4 sm:p-6 pb-3">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center space-x-3">
                                <Skeleton className={cn("h-8 w-8 sm:h-10 sm:w-10 rounded-lg")} />
                                <div className="space-y-1">
                                    <Skeleton className={cn("h-4 w-20 sm:w-24")} />
                                    <Skeleton className={cn("h-3 w-12 sm:w-16")} />
                                </div>
                            </div>
                            <Skeleton className={cn("h-6 w-6 sm:h-8 sm:w-8 rounded")} />
                        </div>
                    </div>
                    <div className="p-4 sm:p-6 pt-0 space-y-3">
                        <Skeleton className={cn("h-3 w-full")} />
                        <Skeleton className={cn("h-3 w-3/4")} />
                        <Skeleton className={cn("h-3 w-1/2")} />
                        <div className="flex flex-wrap gap-2 mt-2">
                            <Skeleton className={cn("h-5 w-12 sm:h-6 sm:w-16 rounded-full")} />
                            <Skeleton className={cn("h-5 w-16 sm:h-6 sm:w-20 rounded-full")} />
                        </div>
                    </div>
                </div>
            ))}
        </div>
    );

    const renderPagination = () => (
        <div className="pt-2">
            {/* 移动端：上一页 + 信息 + 下一页 */}
            <div className="flex sm:hidden items-center justify-between gap-3 h-8">
                <Skeleton className={cn("h-8 flex-1 rounded-md")} />
                <Skeleton className={cn("h-8 w-16 rounded-md")} />
                <Skeleton className={cn("h-8 flex-1 rounded-md")} />
            </div>
            {/* PC端：原有布局 */}
            <div className="hidden sm:flex items-center justify-between gap-4">
                <div className="flex items-center gap-2">
                    <Skeleton className={cn("h-4 w-24")} />
                    <Skeleton className={cn("h-4 w-32")} />
                </div>
                <div className="flex items-center gap-2">
                    <Skeleton className={cn("h-8 w-8 rounded")} />
                    <Skeleton className={cn("h-8 w-8 rounded")} />
                    <Skeleton className={cn("h-8 w-20 rounded")} />
                    <Skeleton className={cn("h-8 w-8 rounded")} />
                    <Skeleton className={cn("h-8 w-8 rounded")} />
                </div>
            </div>
        </div>
    );

    const ariaProps = {
        role: "status" as const,
        "aria-live": "polite" as const,
        "aria-busy": true,
        "aria-label": ariaLabel,
    };

    // 模式 4: 左侧导航栏+顶部过滤栏+卡片列表+分页
    if (variant === 'layout') {
        return (
            <div className={cn("flex", className)} {...ariaProps}>
                <span className="sr-only">{ariaLabel}</span>

                {/* 左侧导航栏骨架 - 移动端隐藏 */}
                <div className="hidden md:block w-64 shrink-0 border-r min-h-[calc(100vh-3rem)]">
                    {/* 导航栏标题 */}
                    <div className="p-4 border-b">
                        <Skeleton className={cn("h-5 w-20")} />
                    </div>

                    {/* 菜单项骨架 */}
                    <div className="p-2 space-y-1">
                        {sidebarPlaceholders.map((_, index) => (
                            <div key={index} className="flex items-center gap-2 px-2 py-2">
                                <Skeleton className={cn("h-4 w-4 shrink-0")} />
                                <Skeleton className={cn("h-4 flex-1")} />
                            </div>
                        ))}
                    </div>
                </div>

                {/* 右侧内容区域 */}
                <div className="flex-1 sm:p-4 space-y-2 sm:space-y-4 min-w-0">
                    {/* 过滤栏骨架 - 移动端隐藏 */}
                    <div className="hidden sm:flex relative flex-wrap items-end gap-2 lg:gap-3 px-4 py-4 sm:pt-5 sm:pb-5 bg-card border rounded-lg shadow-sm">
                        {filterPlaceholders.map((_, index) => (
                            <div key={index} className="flex-1 min-w-[200px] space-y-2">
                                <Skeleton className={cn("h-4 w-20")} />
                                <Skeleton className={cn("h-9 w-full rounded-md")} />
                            </div>
                        ))}

                        <div className="flex gap-2">
                            <Skeleton className={cn("h-9 w-20 rounded-md")} />
                            <Skeleton className={cn("h-9 w-20 rounded-md")} />
                        </div>

                        <div className="absolute top-1.5 right-2 z-10 flex items-center gap-2">
                            <Skeleton className={cn("h-6 w-24")} />
                            <Skeleton className={cn("h-6 w-6 rounded")} />
                        </div>
                    </div>

                    {/* 卡片网格 */}
                    {renderCardGrid()}

                    {/* 分页骨架 */}
                    {renderPagination()}
                </div>
            </div>
        );
    }

    // 模式 3: 带顶部过滤栏+卡片列表+分页
    if (variant === 'page') {
        return (
            <div className={cn("sm:space-y-4", className)} {...ariaProps}>
                <span className="sr-only">{ariaLabel}</span>

                {/* 过滤栏骨架 - 移动端隐藏 */}
                <div className="hidden sm:flex relative flex-wrap items-end gap-2 lg:gap-3 px-4 py-4 sm:pt-5 sm:pb-5 bg-card border rounded-lg shadow-sm">
                    {filterPlaceholders.map((_, index) => (
                        <div key={index} className="flex-1 min-w-[200px] space-y-2">
                            <Skeleton className={cn("h-4 w-20")} />
                            <Skeleton className={cn("h-9 w-full rounded-md")} />
                        </div>
                    ))}

                    <div className="flex gap-2">
                        <Skeleton className={cn("h-9 w-20 rounded-md")} />
                        <Skeleton className={cn("h-9 w-20 rounded-md")} />
                    </div>

                    <div className="absolute top-1.5 right-2 z-10 flex items-center gap-2">
                        <Skeleton className={cn("h-6 w-24")} />
                        <Skeleton className={cn("h-6 w-6 rounded")} />
                    </div>
                </div>

                {/* 卡片网格 */}
                {renderCardGrid()}

                {/* 分页骨架 */}
                {renderPagination()}
            </div>
        );
    }

    // 模式 2: 带分页的完整卡片列表
    if (variant === 'card') {
        return (
            <div className={cn("space-y-4", className)} {...ariaProps}>
                <span className="sr-only">{ariaLabel}</span>

                {/* 卡片网格 */}
                {renderCardGrid()}

                {/* 分页骨架 */}
                {renderPagination()}
            </div>
        );
    }

    // 模式 1: 无边框模式（仅卡片网格）
    return (
        <div
            className={cn(
                'grid gap-4 sm:gap-6 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4',
                className,
            )}
            {...ariaProps}
        >
            <span className="sr-only">{ariaLabel}</span>
            {cardPlaceholders.map((_, index) => (
                <div key={index} className="rounded-lg border bg-card text-card-foreground shadow-sm">
                    <div className="flex flex-col space-y-1.5 p-4 sm:p-6 pb-3">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center space-x-3">
                                <Skeleton className={cn("h-8 w-8 sm:h-10 sm:w-10 rounded-lg")} />
                                <div className="space-y-1">
                                    <Skeleton className={cn("h-4 w-20 sm:w-24")} />
                                    <Skeleton className={cn("h-3 w-12 sm:w-16")} />
                                </div>
                            </div>
                            <Skeleton className={cn("h-6 w-6 sm:h-8 sm:w-8 rounded")} />
                        </div>
                    </div>
                    <div className="p-4 sm:p-6 pt-0 space-y-3">
                        <Skeleton className={cn("h-3 w-full")} />
                        <Skeleton className={cn("h-3 w-3/4")} />
                        <Skeleton className={cn("h-3 w-1/2")} />
                        <div className="flex flex-wrap gap-2 mt-2">
                            <Skeleton className={cn("h-5 w-12 sm:h-6 sm:w-16 rounded-full")} />
                            <Skeleton className={cn("h-5 w-16 sm:h-6 sm:w-20 rounded-full")} />
                        </div>
                    </div>
                </div>
            ))}
        </div>
    );
}
