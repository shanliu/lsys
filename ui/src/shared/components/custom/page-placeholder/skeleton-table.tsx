import { Skeleton } from "@shared/components/ui/skeleton";
import { cn } from "@shared/lib/utils";
import { useIsMobile } from "@shared/hooks/use-mobile";
import { PageSkeletonBaseProps } from "./base";
import { PageSkeletonCard } from "./skeleton-card";

export interface TableSkeletonProps extends PageSkeletonBaseProps {
    /** 表格列数量（0 表示自动计算） */
    columns?: number;
    /** 表格行数量（0 表示自动计算接近满屏） */
    rows?: number;
    /** 
     * 表格骨架屏变体
     * - 'content': 无边框模式（仅内容）
     * - 'table': 带边框模式（完整表格容器）
     * - 'page': 带边框+顶部过滤栏模式（完整页面）
     * - 'layout': 左侧导航栏+带边框+顶部过滤栏模式（完整布局）
     */
    variant?: 'content' | 'table' | 'page' | 'layout';
    /** 左侧导航栏菜单项数量（0 表示自动计算接近满屏） */
    sidebarMenuItems?: number;
}

export function PageSkeletonTable({
    className,
    columns = 0,
    rows = 0,
    variant = 'content',
    sidebarMenuItems = 0,
}: TableSkeletonProps) {
    const isMobile = useIsMobile();

    // 移动端直接显示卡片骨架
    if (isMobile) {
        // 映射 variant: table -> card
        const cardVariant = variant === 'table' ? 'card' : variant;
        return (
            <PageSkeletonCard
                className={className}
                variant={cardVariant}
                sidebarMenuItems={sidebarMenuItems}
            />
        );
    }

    // 以下仅 PC 端渲染

    // 动态计算表格列数
    function getAutoColumns() {
        if (typeof window === 'undefined') return 5;
        const width = window.innerWidth;
        if (width >= 1280) return 6;
        if (width >= 1024) return 5;
        return 4;
    }

    // 动态计算表格行数
    function getAutoRows() {
        if (typeof window === 'undefined') return 6;
        const height = window.innerHeight;
        const rowHeight = 48;
        return Math.ceil((height * 0.5) / rowHeight);
    }

    // 动态计算侧边栏菜单项数
    function getAutoSidebarItems() {
        if (typeof window === 'undefined') return 5;
        const height = window.innerHeight;
        const itemHeight = 44;
        const topSpace = 80;
        return Math.max(1, Math.floor((height - topSpace) / itemHeight));
    }

    const safeColumns = columns > 0 ? columns : getAutoColumns();
    const safeRows = rows > 0 ? rows : getAutoRows();
    const safeSidebarItems = sidebarMenuItems > 0 ? sidebarMenuItems : getAutoSidebarItems();
    const rowPlaceholders = Array.from({ length: safeRows });
    const columnPlaceholders = Array.from({ length: safeColumns });
    const sidebarPlaceholders = Array.from({ length: safeSidebarItems });
    const filterPlaceholders = (variant === 'page' || variant === 'layout') ? Array.from({ length: 3 }) : [];
    const ariaLabel = (variant === 'page' || variant === 'layout') ? "正在加载表格页面..." : "正在加载表格数据...";

    const renderTableRows = (wrapperClassName?: string) => (
        <div className={wrapperClassName}>
            {rowPlaceholders.map((_, rowIndex) => (
                <div key={rowIndex} className="flex items-center py-3 space-x-4">
                    {columnPlaceholders.map((_, columnIndex) => (
                        <div key={columnIndex} className="flex-1">
                            <Skeleton className={cn("h-4 w-full")} />
                        </div>
                    ))}
                </div>
            ))}
        </div>
    );

    const ariaProps = {
        role: "status" as const,
        "aria-live": "polite" as const,
        "aria-busy": true,
        "aria-label": ariaLabel,
    };

    // 模式 4: 左侧导航栏+带边框+顶部过滤栏模式
    if (variant === 'layout') {
        return (
            <div className={cn("flex", className)} {...ariaProps}>
                <span className="sr-only">{ariaLabel}</span>

                {/* 左侧导航栏骨架 */}
                <div className="w-64 shrink-0 border-r min-h-[calc(100vh-3rem)]">
                    {/* 导航栏标题 */}
                    <div className="p-4">
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
                <div className="flex-1 p-4 space-y-4 min-w-0">
                    {/* 过滤栏骨架 */}
                    <div className="relative flex flex-wrap items-end gap-3 px-4 pt-5 pb-5 bg-card border rounded-lg shadow-sm">
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

                    {/* 表格骨架 */}
                    <div className="rounded-lg border bg-card text-card-foreground shadow-sm">
                        <div className="px-6 py-4 border-b border-border/60 bg-muted/30">
                            <div className="flex flex-1 items-center space-x-4">
                                {columnPlaceholders.map((_, index) => (
                                    <div key={index} className="flex-1">
                                        <Skeleton className={cn("h-4 w-full")} />
                                    </div>
                                ))}
                            </div>
                        </div>

                        <div className="px-6 py-2">
                            {renderTableRows("space-y-0 divide-y divide-border/50")}
                        </div>

                        {/* 分页骨架 */}
                        <div className="border-t border-border/60 bg-muted/10 px-6 py-4">
                            <div className="flex items-center justify-between gap-4">
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
                    </div>
                </div>
            </div>
        );
    }

    // 模式 3: 带边框+顶部过滤栏模式
    if (variant === 'page') {
        return (
            <div className={cn("space-y-4", className)} {...ariaProps}>
                <span className="sr-only">{ariaLabel}</span>

                {/* 过滤栏骨架 */}
                <div className="relative flex flex-wrap items-end gap-3 px-4 pt-5 pb-5 bg-card border rounded-lg shadow-sm">
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

                <div className="rounded-lg border bg-card text-card-foreground shadow-sm">
                    <div className="px-6 py-4 border-b border-border/60 bg-muted/30">
                        <div className="flex flex-1 items-center space-x-4">
                            {columnPlaceholders.map((_, index) => (
                                <div key={index} className="flex-1">
                                    <Skeleton className={cn("h-4 w-full")} />
                                </div>
                            ))}
                        </div>
                    </div>

                    <div className="px-6 py-2">
                        {renderTableRows("space-y-0 divide-y divide-border/50")}
                    </div>

                    {/* 分页骨架 */}
                    <div className="border-t border-border/60 bg-muted/10 px-6 py-4">
                        <div className="flex items-center justify-between gap-4">
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
                </div>
            </div>
        );
    }

    // 模式 2: 带边框模式
    if (variant === 'table') {
        return (
            <div className={cn("rounded-lg border bg-card text-card-foreground shadow-sm", className)} {...ariaProps}>
                <span className="sr-only">{ariaLabel}</span>

                <div className="px-6 py-4 border-b border-border/60 bg-muted/30">
                    <div className="flex flex-1 items-center space-x-4">
                        {columnPlaceholders.map((_, index) => (
                            <div key={index} className="flex-1">
                                <Skeleton className={cn("h-4 w-full")} />
                            </div>
                        ))}
                    </div>
                </div>

                <div className="px-6 py-2">
                    {renderTableRows("space-y-0 divide-y divide-border/50")}
                </div>

                {/* 分页骨架 */}
                <div className="border-t border-border/60 bg-muted/10 px-6 py-4">
                    <div className="flex items-center justify-between gap-4">
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
            </div>
        );
    }

    // 模式 1: 无边框模式（仅内容）
    return (
        <div className={cn(className)} {...ariaProps}>
            <span className="sr-only">{ariaLabel}</span>
            {renderTableRows("space-y-2")}
        </div>
    );
}
