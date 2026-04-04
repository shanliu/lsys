/**
 * ExportAction - 通用数据导出面板（纯 UI 组件）
 *
 * 完全不依赖任何具体 API 实现，所有数据和回调通过 props 传入。
 * 业务逻辑（useMutation, useQuery 等）应在父组件中实现。
 *
 * 组件提供：
 * - 移动端："导出" 按钮，点击打开抽屉
 * - 桌面端：分段按钮（左=提交导出，右=查看任务列表）
 * - 任务列表抽屉：展示状态/进度，支持下载文件或外部跳转
 */

import {
  Drawer,
  DrawerContent,
  DrawerHeader,
  DrawerTitle,
} from "@apps/main/components/local/drawer";
import { LayoutParams } from "@apps/main/components/filter-container/container";
import { Button } from "@shared/components/ui/button";
import { Badge } from "@shared/components/ui/badge";
import { cn, formatSeconds, formatTime, TIME_STYLE } from "@shared/lib/utils";

import {
  CheckCircle2,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Clock,
  Download,
  ExternalLink,
  FileDown,
  Loader2,
  XCircle,
} from "lucide-react";
import React, { useState } from "react";

// ─── 数据类型定义 ────────────────────────────────────────────────────────────

/** 导出任务状态 */
export type ExportTaskStatus =
  | "pending"
  | "running"
  | "success"
  | "failed"
  | "deleted";

/** 导出任务数据（UI 友好格式） */
export interface ExportTask {
  id: string | number;
  status: ExportTaskStatus;
  createdAt: Date;
  completedAt?: Date;
  error?: string;
  file?: {
    name: string;
    url: string;
    size: number;
  };
  params?: string; // JSON字符串，用于折叠显示
  type?: string; // 导出类型标签
}

// ─── 状态图标映射 ─────────────────────────────────────────────────────────────

const EXPORT_STATUS_ICONS: Record<ExportTaskStatus, React.ElementType> = {
  pending: Clock,
  running: Loader2,
  success: CheckCircle2,
  failed: XCircle,
  deleted: XCircle,
};

const EXPORT_STATUS_CONFIG: Record<
  ExportTaskStatus,
  { label: string; variant: "neutral" | "info" | "success" | "danger" }
> = {
  pending: { label: "等待中", variant: "neutral" },
  running: { label: "进行中", variant: "info" },
  success: { label: "成功", variant: "success" },
  failed: { label: "失败", variant: "danger" },
  deleted: { label: "已删除", variant: "neutral" },
};

// ─── ExportActionButton - 导出按钮组件（纯UI）──────────────────────────────────

export interface ExportActionButtonProps {
  /** 活跃任务数（显示在按钮角标） */
  activeCount?: number;
  /** 移动端：点击按钮打开抽屉 */
  onClick: () => void;
  /** 桌面端：点击左侧按钮提交导出 */
  onSubmitExport?: () => void;
  /** 桌面端：提交中状态 */
  isSubmitting?: boolean;
  /** 布局参数，用于判断移动端/桌面端 */
  layoutParams?: LayoutParams;
}

export function ExportActionButton({
  activeCount,
  onClick,
  onSubmitExport,
  isSubmitting,
  layoutParams,
}: ExportActionButtonProps) {
  const isMobile = layoutParams?.isMobile;

  if (isMobile) {
    // 移动端：单独的"导出"按钮，点击打开抽屉
    return (
      <Button
        variant="outline"
        size="sm"
        onClick={onClick}
        className="relative"
      >
        <FileDown className="h-4 w-4 mr-1.5" />
        导出
        {activeCount && activeCount > 0 && (
          <Badge
            variant="destructive"
            className="absolute -top-1.5 -right-1.5 h-5 min-w-5 px-1 text-xs flex items-center justify-center"
          >
            {activeCount}
          </Badge>
        )}
      </Button>
    );
  }

  // 桌面端：分段按钮：左=提交导出，右=查看任务列表
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
      {/* 右段：查看任务列表 */}
      <button
        type="button"
        onClick={onClick}
        className="flex items-center gap-1 pl-2 pr-2.5 hover:bg-accent hover:text-accent-foreground transition-colors text-muted-foreground font-medium"
      >
        <Clock className="h-3.5 w-3.5 flex-shrink-0" />
        {activeCount && activeCount > 0 && (
          <span className="text-xs tabular-nums">{activeCount}</span>
        )}
      </button>
    </div>
  );
}

// ─── ExportActionDrawer - 导出抽屉组件（纯UI）─────────────────────────────────

export interface ExportActionDrawerProps {
  /** 抽屉是否打开 */
  open: boolean;
  /** 抽屉打开/关闭状态变化回调 */
  onOpenChange: (open: boolean) => void;
  /** 任务列表数据 */
  tasks: ExportTask[];
  /** 总任务数 */
  totalCount: number;
  /** 当前页码 */
  currentPage: number;
  /** 总页数 */
  totalPages: number;
  /** 页码变化回调 */
  onPageChange: (page: number) => void;
  /** 是否加载中 */
  isLoading?: boolean;
  /** 点击"查看文件列表"回调 */
  onViewFile?: (taskId: number) => void;
  /** 移动端：提交导出回调 */
  onSubmitExport?: () => void;
  /** 移动端：提交中状态 */
  isSubmitting?: boolean;
  /** 抽屉标题 */
  drawerTitle?: React.ReactNode;
  /** 布局参数 */
  layoutParams?: LayoutParams;
}

export function ExportActionDrawer({
  open,
  onOpenChange,
  tasks,
  totalCount,
  currentPage,
  totalPages,
  onPageChange,
  isLoading,
  onViewFile,
  onSubmitExport,
  isSubmitting,
  drawerTitle,
  layoutParams,
}: ExportActionDrawerProps) {
  const [expandedParams, setExpandedParams] = useState<Set<number | string>>(
    new Set(),
  );
  const isMobile = layoutParams?.isMobile;

  const toggleParams = (id: number | string) =>
    setExpandedParams((prev) => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });

  const tryFormatJson = (raw: string) => {
    try {
      return JSON.stringify(JSON.parse(raw), null, 2);
    } catch {
      return raw;
    }
  };

  // 将任务分为"正在进行的任务"和"历史记录"
  const activeTasks = tasks.filter(
    (t) => t.status === "pending" || t.status === "running",
  );
  const historyTasks = tasks.filter(
    (t) => t.status !== "pending" && t.status !== "running",
  );

  const renderTaskItem = (task: ExportTask) => {
    const config = EXPORT_STATUS_CONFIG[task.status];
    const StatusIcon = EXPORT_STATUS_ICONS[task.status];
    const isProcessing = task.status === "pending" || task.status === "running";
    const isDone = task.status === "success";

    return (
      <div
        key={task.id}
        className={cn(
          "rounded-lg border p-3 space-y-2",
          isDone &&
            "border-green-200 bg-green-50/50 dark:border-green-900 dark:bg-green-950/20",
          task.status === "failed" && "border-destructive/30 bg-destructive/5",
        )}
      >
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 min-w-0 flex-wrap">
            <span className="text-xs text-muted-foreground flex-shrink-0">
              #{task.id}
            </span>
            {task.type && (
              <Badge
                variant="outline"
                className="text-xs px-1.5 py-0 flex-shrink-0 font-mono"
              >
                {task.type}
              </Badge>
            )}
            <Badge
              className={cn(
                "text-xs flex items-center gap-1 flex-shrink-0",
                config.variant === "success" &&
                  "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
                config.variant === "danger" &&
                  "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
                config.variant === "info" &&
                  "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                config.variant === "neutral" &&
                  "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
              )}
            >
              <StatusIcon
                className={cn("h-3 w-3", isProcessing && "animate-spin")}
              />
              {config.label}
            </Badge>
          </div>
          <span className="text-xs text-muted-foreground flex-shrink-0">
            {task.createdAt
              ? formatTime(task.createdAt, TIME_STYLE.RELATIVE_ELEMENT)
              : "-"}
          </span>
        </div>

        {/* 耗时 */}
        {isDone &&
          !!task.completedAt &&
          task.createdAt &&
          task.completedAt.getTime() > task.createdAt.getTime() && (
            <div className="text-xs text-muted-foreground">
              耗时：
              {formatSeconds(
                task.completedAt.getTime() - task.createdAt.getTime(),
              )}
            </div>
          )}

        {/* 失败原因 */}
        {task.status === "failed" && task.error && (
          <div className="text-xs text-destructive break-all">
            <span className="font-medium">失败原因：</span>
            {task.error}
          </div>
        )}

        {/* 操作按钮 */}
        {isDone && (task.file?.url || onViewFile) && (
          <div className="flex justify-end gap-2 pt-1">
            {task.file?.url && (
              <Button
                size="sm"
                variant="default"
                className="h-7 text-xs"
                asChild
              >
                <a
                  href={task.file.url}
                  download={task.file.name}
                  target="_blank"
                  rel="noreferrer"
                >
                  <FileDown className="h-3 w-3 mr-1" />
                  下载文件
                </a>
              </Button>
            )}
            {onViewFile && (
              <Button
                size="sm"
                variant="outline"
                className="h-7 text-xs"
                onClick={() => onViewFile(Number(task.id))}
              >
                <ExternalLink className="h-3 w-3 mr-1" />
                查看文件列表
              </Button>
            )}
          </div>
        )}

        {/* 导出条件：折叠展示 JSON */}
        {task.params && (
          <div className="border-t pt-1.5 mt-0.5">
            <button
              type="button"
              onClick={() => toggleParams(task.id)}
              className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors w-full text-left"
            >
              {expandedParams.has(task.id) ? (
                <ChevronDown className="h-3 w-3 flex-shrink-0" />
              ) : (
                <ChevronRight className="h-3 w-3 flex-shrink-0" />
              )}
              条件
            </button>
            {expandedParams.has(task.id) && (
              <pre className="mt-1.5 text-xs bg-muted rounded p-2 overflow-x-auto leading-relaxed whitespace-pre-wrap break-all">
                {tryFormatJson(task.params)}
              </pre>
            )}
          </div>
        )}
      </div>
    );
  };

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader className="pb-4">
          <DrawerTitle className="text-xl">
            {drawerTitle ?? "导出任务"}
          </DrawerTitle>
        </DrawerHeader>

        <div className="px-4 pb-6 space-y-3 overflow-y-auto">
          {/* 移动端：在抽屉顶部显示"创建导出任务"按钮 */}
          {isMobile && onSubmitExport && (
            <div className="pb-3 border-b">
              <Button
                onClick={onSubmitExport}
                disabled={isSubmitting}
                className="w-full"
              >
                {isSubmitting ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    提交中...
                  </>
                ) : (
                  <>
                    <FileDown className="h-4 w-4 mr-2" />
                    创建导出任务
                  </>
                )}
              </Button>
            </div>
          )}

          {isLoading ? (
            <div className="flex items-center justify-center py-10 text-muted-foreground">
              <Loader2 className="h-5 w-5 animate-spin mr-2" />
              加载中...
            </div>
          ) : tasks.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-10 text-muted-foreground">
              <Download className="h-10 w-10 mb-3 opacity-30" />
              <p className="text-sm">暂无导出任务</p>
              <p className="text-xs mt-1 opacity-70">
                按条件过滤后点击"导出"可创建任务
              </p>
            </div>
          ) : (
            <>
              {/* 正在进行的任务 */}
              {activeTasks.length > 0 && (
                <div className="space-y-2">
                  <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    正在进行的任务
                  </div>
                  <div className="space-y-3">
                    {activeTasks.map(renderTaskItem)}
                  </div>
                </div>
              )}

              {/* 历史记录 */}
              {historyTasks.length > 0 && (
                <div className="space-y-2">
                  {activeTasks.length > 0 && <div className="border-t my-4" />}
                  <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
                    <Clock className="h-4 w-4" />
                    历史记录
                  </div>
                  <div className="space-y-3">
                    {historyTasks.map(renderTaskItem)}
                  </div>
                </div>
              )}

              {/* 分页控件 */}
              {totalPages > 1 && (
                <div className="flex justify-center items-center gap-2 pt-4 border-t">
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={currentPage === 1}
                    onClick={() => onPageChange(currentPage - 1)}
                  >
                    <ChevronLeft className="h-4 w-4" />
                    上一页
                  </Button>
                  <span className="text-sm text-muted-foreground px-2">
                    第 {currentPage} / {totalPages} 页（共 {totalCount} 条）
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={currentPage === totalPages}
                    onClick={() => onPageChange(currentPage + 1)}
                  >
                    下一页
                    <ChevronRight className="h-4 w-4" />
                  </Button>
                </div>
              )}
            </>
          )}
        </div>
      </DrawerContent>
    </Drawer>
  );
}

// ─── ExportAction - 主组件（组合按钮和抽屉）───────────────────────────────────

export interface ExportActionProps {
  /** 活跃任务数（显示在按钮角标） */
  activeCount?: number;
  /** 点击按钮回调（移动端：打开抽屉；桌面端：查看任务列表） */
  onButtonClick: () => void;

  /** 抽屉是否打开 */
  drawerOpen: boolean;
  /** 抽屉打开/关闭状态变化回调 */
  onDrawerOpenChange: (open: boolean) => void;
  /** 任务列表数据 */
  tasks: ExportTask[];
  /** 总任务数 */
  totalCount: number;
  /** 当前页码 */
  currentPage: number;
  /** 总页数 */
  totalPages: number;
  /** 页码变化回调 */
  onPageChange: (page: number) => void;
  /** 是否加载中 */
  isLoading?: boolean;
  /** 点击"查看文件列表"回调 */
  onViewFile?: (taskId: number) => void;

  /** 提交导出回调 */
  onSubmitExport?: () => void;
  /** 提交中状态 */
  isSubmitting?: boolean;

  /** 抽屉标题 */
  drawerTitle?: React.ReactNode;
  /** 布局参数 */
  layoutParams?: LayoutParams;
}

export function ExportAction({
  activeCount,
  onButtonClick,
  drawerOpen,
  onDrawerOpenChange,
  tasks,
  totalCount,
  currentPage,
  totalPages,
  onPageChange,
  isLoading,
  onViewFile,
  onSubmitExport,
  isSubmitting,
  drawerTitle,
  layoutParams,
}: ExportActionProps) {
  return (
    <>
      <ExportActionButton
        activeCount={activeCount}
        onClick={onButtonClick}
        onSubmitExport={onSubmitExport}
        isSubmitting={isSubmitting}
        layoutParams={layoutParams}
      />
      <ExportActionDrawer
        open={drawerOpen}
        onOpenChange={onDrawerOpenChange}
        tasks={tasks}
        totalCount={totalCount}
        currentPage={currentPage}
        totalPages={totalPages}
        onPageChange={onPageChange}
        isLoading={isLoading}
        onViewFile={onViewFile}
        onSubmitExport={layoutParams?.isMobile ? onSubmitExport : undefined}
        isSubmitting={isSubmitting}
        drawerTitle={drawerTitle}
        layoutParams={layoutParams}
      />
    </>
  );
}
