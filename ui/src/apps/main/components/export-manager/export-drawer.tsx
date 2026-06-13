import {
  Drawer,
  DrawerContent,
  DrawerHeader,
  DrawerTitle,
} from "@apps/main/components/local/drawer";
import { PostDownload } from "@apps/main/components/local/post-download";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { cn, formatSeconds, formatTime, TIME_STYLE } from "@shared/lib/utils";
import { DictList } from "@shared/types/apis-dict";
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

// ─── 数据类型 ─────────────────────────────────────────────────────────────────

/** 导出任务数据（UI 友好格式，由各 export hook 从 API 响应映射而来） */
export interface ExportTask {
  id: string | number;
  /** 后端状态码：1=等待 2=进行中 3=成功 4=失败 5=已删除 */
  status: number;
  createdAt: Date;
  completedAt?: Date;
  error?: string;
  /** 文件下载 POST URL（通过 authApi POST 下载，避免 GET 无法携带 token）*/
  downloadPostUrl?: string;
  /** JSON 字符串，用于折叠显示导出条件 */
  params?: string;
  /** 导出类型标签 */
  type?: string;
}

// ─── 状态映射（纯 UI 逻辑，不依赖字典） ──────────────────────────────────────
// 1=等待 2=进行中 3=成功 4=失败 5=已删除

const EXPORT_STATUS_ICONS: Record<number, React.ElementType> = {
  1: Clock,
  2: Loader2,
  3: CheckCircle2,
  4: XCircle,
  5: XCircle,
};

const EXPORT_STATUS_VARIANT: Record<
  number,
  "neutral" | "info" | "success" | "danger"
> = {
  1: "neutral",
  2: "info",
  3: "success",
  4: "danger",
  5: "neutral",
};

// ─── ExportDrawer ─────────────────────────────────────────────────────────────

export interface ExportDrawerProps {
  /** 抽屉是否打开 */
  open: boolean;
  /** 抽屉打开/关闭状态变化回调 */
  onOpenChange: (open: boolean) => void;
  /** 状态字典（来自 admin_export 或 user_export dict 的 export_task_status 字段） */
  statusDict: DictList;
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
  /** 抽屉标题 */
  drawerTitle?: React.ReactNode;
}

export function ExportDrawer({
  open,
  onOpenChange,
  statusDict,
  tasks,
  totalCount,
  currentPage,
  totalPages,
  onPageChange,
  isLoading,
  onViewFile,
  drawerTitle,
}: ExportDrawerProps) {
  const [expandedParams, setExpandedParams] = useState<Set<number | string>>(
    new Set(),
  );

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

  const activeTasks = tasks.filter((t) => t.status === 1 || t.status === 2);
  const historyTasks = tasks.filter((t) => t.status !== 1 && t.status !== 2);

  const renderTaskItem = (task: ExportTask) => {
    const statusVariant = EXPORT_STATUS_VARIANT[task.status] ?? "neutral";
    const StatusIcon = EXPORT_STATUS_ICONS[task.status] ?? Clock;
    const isProcessing = task.status === 1 || task.status === 2;
    const isDone = task.status === 3;

    return (
      <div
        key={task.id}
        className={cn(
          "rounded-lg border p-3 space-y-2",
          isDone &&
            "border-green-200 bg-green-50/50 dark:border-green-900 dark:bg-green-950/20",
          task.status === 4 && "border-destructive/30 bg-destructive/5",
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
                statusVariant === "success" &&
                  "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
                statusVariant === "danger" &&
                  "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
                statusVariant === "info" &&
                  "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                statusVariant === "neutral" &&
                  "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
              )}
            >
              <StatusIcon
                className={cn("h-3 w-3", isProcessing && "animate-spin")}
              />
              {statusDict.getLabel(String(task.status))}
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
        {task.status === 4 && task.error && (
          <div className="text-xs text-destructive break-all">
            <span className="font-medium">失败原因：</span>
            {task.error}
          </div>
        )}

        {/* 操作按钮 */}
        {isDone && (task.downloadPostUrl || onViewFile) && (
          <div className="flex justify-end gap-2 pt-1">
            {task.downloadPostUrl && (
              <PostDownload
                url={task.downloadPostUrl}
                body={{ task_id: task.id }}
              >
                {({ onClick, isLoading, error }) => (
                  <>
                    <Button
                      size="sm"
                      variant="default"
                      className="h-7 text-xs"
                      onClick={onClick}
                      disabled={isLoading}
                    >
                      {isLoading ? (
                        <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                      ) : (
                        <FileDown className="h-3 w-3 mr-1" />
                      )}
                      {isLoading ? "下载中..." : "下载文件"}
                    </Button>
                    {error && (
                      <p className="text-xs text-destructive mt-1 break-all">
                        {error}
                      </p>
                    )}
                  </>
                )}
              </PostDownload>
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
