import { Button } from "@shared/components/ui/button";
import { Clock, FileDown, History, Loader2 } from "lucide-react";
import React from "react";

// ─── ExportMobileButton ───────────────────────────────────────────────────────
// 放置于 FilterBar 的 mobileHeaderActions，点击打开导出历史抽屉。

export interface ExportMobileButtonProps {
  /** 点击打开导出抽屉 */
  onClick: () => void;
  /** 活跃任务数（角标） */
  activeCount?: number;
  /** 是否有正在进行的任务（显示旋转动画） */
  isLoading?: boolean;
}

export function ExportMobileButton({
  onClick,
  activeCount,
  isLoading,
}: ExportMobileButtonProps) {
  return (
    <Button variant="ghost" size="sm" className="h-8 px-3" onClick={onClick}>
      <History className="h-4 w-4 mr-2" />
      导出
      {!!activeCount && activeCount > 0 && (
        <span className="ml-1.5 flex h-5 w-5 items-center justify-center rounded-full border border-current text-xs font-medium">
          {activeCount}
        </span>
      )}
      {isLoading && <Loader2 className="ml-1.5 h-3.5 w-3.5 animate-spin" />}
    </Button>
  );
}

// ─── ExportButton ─────────────────────────────────────────────────────────────
// 桌面端分段按钮：左=提交导出，右=查看历史（角标）。
// 放置于 FilterBar children 的过滤字段区域。

export interface ExportSplitButtonProps {
  /** 点击左侧提交导出 */
  onSubmitExport?: () => void | Promise<void>;
  /** 点击右侧查看历史抽屉 */
  onViewHistory: () => void;
  /** 提交中状态 */
  isSubmitting?: boolean;
  /** 活跃任务数（右侧角标） */
  activeCount?: number;
}

export function ExportSplitButton({
  onSubmitExport,
  onViewHistory,
  isSubmitting,
  activeCount,
}: ExportSplitButtonProps) {
  return (
    <div className="flex h-10 border border-input rounded-md shadow-sm overflow-hidden bg-background text-sm">
      {/* 左段：提交导出 */}
      <button
        type="button"
        disabled={isSubmitting}
        onClick={onSubmitExport}
        className="flex items-center gap-1.5 pl-3 pr-3 hover:bg-accent hover:text-accent-foreground transition-colors disabled:pointer-events-none disabled:opacity-50 font-medium"
      >
        {isSubmitting ? (
          <Loader2 className="h-4 w-4 animate-spin" />
        ) : (
          <FileDown className="h-4 w-4" />
        )}
        导出
      </button>
      {/* 分隔线 */}
      <span className="w-px bg-input self-stretch" />
      {/* 右段：查看任务历史 */}
      <button
        type="button"
        onClick={onViewHistory}
        className="flex items-center gap-1 pl-2 pr-2.5 hover:bg-accent hover:text-accent-foreground transition-colors text-muted-foreground font-medium"
      >
        <Clock className="h-3.5 w-3.5 flex-shrink-0" />
        {!!activeCount && activeCount > 0 && (
          <span className="text-xs tabular-nums">{activeCount}</span>
        )}
      </button>
    </div>
  );
}

// ─── ExportButton ────────────────────────────────────────────────────────────────────
// 通用导出按钮：仅提交导出。移动端用于 FilterBar.MobileFooter。

export interface ExportButtonProps {
  /** 提交导出（通常已包含关闭抽屉逻辑） */
  onSubmitExport: () => void;
  /** 提交中状态 */
  isSubmitting?: boolean;
}

export function ExportButton({
  onSubmitExport,
  isSubmitting,
}: ExportButtonProps) {
  return (
    <Button
      type="button"
      variant="outline"
      className="w-full"
      disabled={isSubmitting}
      onClick={onSubmitExport}
    >
      {isSubmitting ? (
        <Loader2 className="h-4 w-4 animate-spin" />
      ) : (
        <FileDown className="h-4 w-4" />
      )}
      导出
    </Button>
  );
}
