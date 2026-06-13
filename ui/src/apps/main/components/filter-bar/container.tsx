import {
  Drawer,
  DrawerContent,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from "@apps/main/components/local/drawer";
import { Button } from "@shared/components/ui/button";
import { useIsMobile } from "@shared/hooks/use-mobile";
import { cn } from "@shared/lib/utils";
import {
  ChevronDown,
  ChevronUp,
  Filter,
  MoreHorizontal,
} from "lucide-react";
import React from "react";
import { FieldValues, FormProvider } from "react-hook-form";
import ReactDOM from "react-dom";
import {
  FilterBarPortalContext,
  FilterFieldContext,
  useFilterBarPortalContext,
  useFilterFieldContext,
  type FilterBarPortalContextValue,
  type LayoutParams,
  type UseFilterFormReturn,
} from "./context";

// ─── MobileFooter children type ──────────────────────────────────────────────
/** MobileFooter children: plain node or render prop receiving closeMobilePanel */
export type MobileFooterChildren =
  | React.ReactNode
  | ((closeMobilePanel: () => void) => React.ReactNode);

// ─── FilterBar props ───────────────────────────────────────────────────────────
export interface FilterBarProps<
  TFieldValues extends FieldValues = FieldValues,
> {
  /**
   * 由 useFilterForm() hook 创建的表单实例。
   * 表单状态由页面持有，FilterBar 负责提供 FormProvider + FilterFieldContext。
   */
  form: UseFilterFormReturn<TFieldValues>;
  /**
   * 子组件：普通子元素为 filter fields。
   * FilterBar.Summary / .MobileExtra / .MobileFooter 是真正的 React 组件，
   * 它们使用 portal 将内容渲染到 FilterBar 提供的目标位置。
   * 父组件对这些子组件类型零感知——无 Children.forEach，无 Symbol 检测。
   */
  children: React.ReactNode;
  className?: string;
}

// ─── FilterBar ────────────────────────────────────────────────────────────────

/**
 * FilterBar — 过滤器栏组件
 *
 * 职责：
 *   1. 提供 FormProvider（RHF）和 FilterFieldContext，让 filter 字段子组件无需 prop 透传
 *   2. 提供 FilterBarPortalContext，包含三个 portal 目标 DOM 节点
 *   3. 移动端：触发按钮 + Drawer 内容
 *   4. 桌面端：内联卡片 + 可折叠
 *
 * 表单状态由外部 useFilterForm() 管理，FilterBar 只持有 UI 状态（drawer open、collapsed）。
 */
function FilterBarComponent<TFieldValues extends FieldValues = FieldValues>({
  form,
  children,
  className,
}: FilterBarProps<TFieldValues>) {
  const isMobile = useIsMobile();
  const [isMobileOpen, setIsMobileOpen] = React.useState(false);
  const [isCollapsed, setIsCollapsed] = React.useState(false);

  const closeMobilePanel = React.useCallback(() => setIsMobileOpen(false), []);

  // ── Portal targets ────────────────────────────────────────────────────────
  // useState setter 直接作为 ref callback 使用：
  // div 挂载时 setter 触发，引起一次额外 re-render，portal 目标变为 HTMLElement。
  // 这是 React portal 的标准模式，额外 re-render 在挂载时只发生一次，视觉无感知。
  const [summaryPortal, setSummaryPortal] =
    React.useState<HTMLElement | null>(null);
  const [mobileExtraPortal, setMobileExtraPortal] =
    React.useState<HTMLElement | null>(null);
  const [mobileFooterPortal, setMobileFooterPortal] =
    React.useState<HTMLElement | null>(null);

  // ── Contexts ──────────────────────────────────────────────────────────────
  const layoutParams: LayoutParams = React.useMemo(
    () => ({ isMobile }),
    [isMobile],
  );

  // 增强 handleFormSubmit：提交后自动关闭移动端面板
  const contextForm = React.useMemo(
    () =>
      ({
        ...form,
        handleFormSubmit: async (e?: React.BaseSyntheticEvent) => {
          await form.handleFormSubmit(e);
          setIsMobileOpen(false);
        },
      }) as UseFilterFormReturn<TFieldValues>,
    [form],
  );

  const filterContextValue = React.useMemo(
    () => ({ layoutParams, form: contextForm }),
    [layoutParams, contextForm],
  );

  const filterBarPortalContextValue: FilterBarPortalContextValue = React.useMemo(
    () => ({
      summaryPortal,
      mobileExtraPortal,
      mobileFooterPortal,
      closeMobilePanel,
    }),
    [summaryPortal, mobileExtraPortal, mobileFooterPortal, closeMobilePanel],
  );

  // 移动端 Drawer 内部使用 no-op portal context：
  // children 在 Drawer 内渲染只用于显示 filter fields；
  // portal 组件（Summary/MobileExtra/MobileFooter）在此 context 下 portal 目标为 null → 返回 null，
  // 避免与下方始终挂载的隐藏副本产生重复渲染。
  const noOpPortalContextValue: FilterBarPortalContextValue = React.useMemo(
    () => ({
      summaryPortal: null,
      mobileExtraPortal: null,
      mobileFooterPortal: null,
      closeMobilePanel,
    }),
    [closeMobilePanel],
  );

  // ── Mobile layout ─────────────────────────────────────────────────────────
  if (isMobile) {
    return (
      <FilterBarPortalContext.Provider value={filterBarPortalContextValue}>
        <FilterFieldContext.Provider value={filterContextValue}>
          <FormProvider {...form}>
            {/*
              隐藏副本：始终挂载在 DOM 中（不受 Drawer 开关状态影响）。
              作用：让 FilterBar.Summary / .MobileExtra / .MobileFooter 的 portal 始终能触发，
              使 header 上的汇总信息和导出按钮在抽屉关闭时也正常显示。
              使用 position:absolute + overflow:hidden 使其不占布局空间、不可交互。
            */}
            <div
              aria-hidden="true"
              style={{
                position: "absolute",
                width: 0,
                height: 0,
                overflow: "hidden",
                pointerEvents: "none",
              }}
            >
              {children}
            </div>

            <div className="flex items-center justify-between gap-2">
              {/* Summary portal target: 移动端 header 左侧 */}
              <div
                className="flex-1 min-w-0"
                ref={setSummaryPortal as React.RefCallback<HTMLDivElement>}
              />

              <div className="flex items-center gap-1 flex-shrink-0">
                {/*
                  Drawer 内使用 no-op portal context：
                  children 在此只负责渲染可见的 filter fields；
                  portal 组件在 noOpPortalContext 下 target 为 null → 返回 null，不重复注入。
                */}
                <FilterBarPortalContext.Provider value={noOpPortalContextValue}>
                  <Drawer open={isMobileOpen} onOpenChange={setIsMobileOpen}>
                    <DrawerTrigger asChild>
                      <Button variant="ghost" size="sm" className={cn("h-8 px-3")}>
                        <Filter className={cn("h-4 w-4 mr-2")} />
                        筛选
                        {form.filledFilterCount > 0 && (
                          <span className="ml-1.5 flex h-5 w-5 items-center justify-center rounded-full border border-current text-xs font-medium">
                            {form.filledFilterCount}
                          </span>
                        )}
                      </Button>
                    </DrawerTrigger>

                    <DrawerContent
                      className={cn("w-[95%] sm:max-w-md")}
                      contentClassName="p-4"
                    >
                      <DrawerHeader className={cn("pb-4")}>
                        <DrawerTitle
                          className={cn("flex items-center gap-2 text-left")}
                        >
                          <Filter className={cn("h-4 w-4")} />
                          筛选条件
                        </DrawerTitle>
                      </DrawerHeader>

                      {/* Filter fields: 渲染在 Drawer 内容区 */}
                      <div className="flex flex-col gap-4 overflow-y-auto">
                        {children}
                      </div>

                      {/* MobileFooter portal target: 移动端面板底部（由隐藏副本的 MobileFooter portal 注入内容） */}
                      <div
                        ref={
                          setMobileFooterPortal as React.RefCallback<HTMLDivElement>
                        }
                      />
                    </DrawerContent>
                  </Drawer>
                </FilterBarPortalContext.Provider>

                {/* MobileExtra portal target: 筛选按钮右侧 */}
                <div
                  ref={
                    setMobileExtraPortal as React.RefCallback<HTMLDivElement>
                  }
                />
              </div>
            </div>
          </FormProvider>
        </FilterFieldContext.Provider>
      </FilterBarPortalContext.Provider>
    );
  }

  // ── Desktop layout ────────────────────────────────────────────────────────
  return (
    <FilterBarPortalContext.Provider value={filterBarPortalContextValue}>
      <FilterFieldContext.Provider value={filterContextValue}>
        <FormProvider {...form}>
          {isCollapsed ? (
            <div
              className={cn(
                "relative flex items-center justify-center gap-2 px-4 py-2 bg-card border rounded-lg shadow-sm cursor-pointer hover:bg-accent/50 transition-colors",
                className,
              )}
              onClick={() => setIsCollapsed(false)}
            >
              <MoreHorizontal
                className={cn("h-4 w-4 text-muted-foreground")}
              />
              <span className="text-xs text-muted-foreground font-medium">
                当前筛选
              </span>
              {/* Summary portal target: 折叠状态内联显示 */}
              <div
                className="[&>*]:border-0 [&>*]:bg-transparent [&>*]:shadow-none"
                ref={setSummaryPortal as React.RefCallback<HTMLDivElement>}
              />
              <ChevronDown
                className={cn("h-4 w-4 text-muted-foreground ml-1")}
              />
            </div>
          ) : (
            <div
              className={cn(
                "relative flex flex-col gap-3 px-4 pt-5 pb-5 bg-card border rounded-lg shadow-sm",
                className,
              )}
            >
              <div className={cn("flex flex-wrap items-end gap-2 lg:gap-3")}>
                {children}
              </div>

              {/* 右上角：summary portal target + 折叠按钮 */}
              <div className="absolute top-1.5 right-2 z-10 flex items-center gap-2">
                <div
                  ref={setSummaryPortal as React.RefCallback<HTMLDivElement>}
                />
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn("h-6 w-6 p-0 hover:bg-accent")}
                  onClick={() => setIsCollapsed(true)}
                >
                  <ChevronUp className={cn("h-3 w-3")} />
                </Button>
              </div>
              {/* mobileExtraPortal / mobileFooterPortal 在桌面端不设置
                  → FilterBar.MobileExtra / .MobileFooter 的 ctx.xxxPortal 为 null
                  → 它们直接 return null，桌面端不渲染这两个区域 */}
            </div>
          )}
        </FormProvider>
      </FilterFieldContext.Provider>
    </FilterBarPortalContext.Provider>
  );
}

// ─── FilterBar = component + static sub-components ──────────────────────────
export const FilterBar = Object.assign(FilterBarComponent, {
  Summary: ({ children }: { children: React.ReactNode }) => {
    const { summaryPortal } = useFilterBarPortalContext();
    if (!summaryPortal) return null;
    return ReactDOM.createPortal(children, summaryPortal);
  },
  MobileExtra: ({ children }: { children: React.ReactNode }) => {
    const { mobileExtraPortal } = useFilterBarPortalContext();
    if (!mobileExtraPortal) return null;
    return ReactDOM.createPortal(children, mobileExtraPortal);
  },
  MobileFooter: ({ children }: { children: MobileFooterChildren }) => {
    const { mobileFooterPortal, closeMobilePanel } = useFilterBarPortalContext();
    if (!mobileFooterPortal) return null;
    const content =
      typeof children === "function" ? children(closeMobilePanel) : children;
    return ReactDOM.createPortal(
      <div className="mt-4 pt-3 border-t">{content}</div>,
      mobileFooterPortal,
    );
  },
  /** 仅在桌面端渲染，移动端返回 null。与 MobileExtra/MobileFooter 的自隐藏风格一致。 */
  DesktopOnly: ({ children }: { children: React.ReactNode }) => {
    const { layoutParams } = useFilterFieldContext();
    if (layoutParams.isMobile) return null;
    return <>{children}</>;
  },
});


