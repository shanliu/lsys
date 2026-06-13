import React from "react";
import type { FieldValues, UseFormReturn } from "react-hook-form";

// ─── 核心类型 ─────────────────────────────────────────────────────────────────

export interface LayoutParams {
  isMobile: boolean;
}

/** 扩展的 form 类型，在标准 RHF form 基础上增加预配置的提交/重置方法 */
export interface ExtendedFormReturn<
  TFieldValues extends FieldValues = FieldValues,
> extends UseFormReturn<TFieldValues> {
  handleFormSubmit: (e?: React.BaseSyntheticEvent) => Promise<void>;
  handleFormReset: () => void;
}

/** useFilterForm hook 的返回类型，在 ExtendedFormReturn 基础上增加已填写条件计数 */
export interface UseFilterFormReturn<
  TFieldValues extends FieldValues = FieldValues,
> extends ExtendedFormReturn<TFieldValues> {
  /** 当前已填写的筛选条件数量，用于移动端徽标显示 */
  filledFilterCount: number;
}

// ─── FilterFieldContext ──────────────────────────────────────────────────────
//
// 让 FilterBar 内所有字段子组件可以通过 useFilterFieldContext() 直接读取：
//   - layoutParams（isMobile），不再需要 prop 手动透传
//   - form，FilterActions 等不再需要接受 form prop

export interface FilterFieldContextValue {
  layoutParams: LayoutParams;
  form: UseFilterFormReturn<any>;
}

export const FilterFieldContext =
  React.createContext<FilterFieldContextValue | null>(null);

/**
 * 在 FilterBar 内部使用，获取布局参数、form 和关闭移动端面板方法。
 * 必须在 <FilterBar> 内部调用，否则抛出错误。
 */
export function useFilterFieldContext(): FilterFieldContextValue {
  const ctx = React.useContext(FilterFieldContext);
  if (!ctx) {
    throw new Error("useFilterFieldContext 必须在 <FilterBar> 内部使用");
  }
  return ctx;
}

// ─── FilterBarPortalContext ───────────────────────────────────────────────────
//
// FilterBar 内部的 portal 目标位置 + 辅助状态。
// 子组件（FilterBar.Summary / .MobileExtra / .MobileFooter）通过此 context
// 获取各自的 portal 目标 DOM 节点，然后用 createPortal 把内容渲染到正确位置。
// React context 可以跨 portal 传播，所以 FilterFieldContext / FormProvider 对 portal 内容仍然有效。

export interface FilterBarPortalContextValue {
  /** 总数/摘要区域的 portal 目标（移动端：header 左侧；桌面端：卡片右上角绝对位置） */
  summaryPortal: HTMLElement | null;
  /** 移动端 header 额外操作的 portal 目标（桌面端不设置 → null → 不渲染） */
  mobileExtraPortal: HTMLElement | null;
  /** 移动端筛选面板底部的 portal 目标（桌面端不设置 → null → 不渲染） */
  mobileFooterPortal: HTMLElement | null;
  /** 关闭移动端筛选面板 */
  closeMobilePanel: () => void;
}

export const FilterBarPortalContext =
  React.createContext<FilterBarPortalContextValue | null>(null);

export function useFilterBarPortalContext(): FilterBarPortalContextValue {
  const ctx = React.useContext(FilterBarPortalContext);
  if (!ctx) {
    throw new Error("useFilterBarPortalContext 必须在 <FilterBar> 内部使用");
  }
  return ctx;
}
