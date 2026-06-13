/**
 * useUserAppExportAction — 应用级导出操作 hook（需要 appId）
 *
 * 适用于特定应用内的导出（如应用文件列表、应用邮件、应用短信等），
 * 必须传入 appId，使用 /api/user/app_export_task/* 系列接口。
 *
 * 返回纯数据 + 操作函数，调用方负责渲染 UI。
 * 示例：
 *   const exp = useUserAppExportAction({
 *     appId,
 *     exportType,
 *     params,
 *     onViewFile: (taskId) => navigate({ to: "...", search: { tag_name: `export_${taskId}` } }),
 *   });
 *   <ExportMobileButton activeCount={exp.activeCount} onClick={exp.openDrawer} />
 *   <ExportSplitButton onSubmitExport={exp.submit} onViewHistory={exp.openDrawer} ... />
 *   <ExportDrawer open={exp.drawerOpen} onViewFile={exp.handleViewFile} ... tasks={exp.tasks} ... />
 */

import { type ExportTask } from "@apps/main/components/export-manager";
import { type UseExportActionResult } from "@apps/main/hooks/use-user-export-action";
import {
  userFileExportActiveCount,
  userFileExportList,
  userFileExportSubmit,
} from "@shared/apis/user/file";
import { useToast } from "@shared/contexts/toast-context";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

export type { UseExportActionResult } from "@apps/main/hooks/use-user-export-action";

export interface UseUserAppExportActionOptions {
  /** 应用 ID */
  appId: number;
  /** 导出类型常量 */
  exportType: string;
  params?: Record<string, unknown>;
  /**
   * 点击"查看文件列表"的回调。
   * 传入时 hook 返回 handleViewFile；不传时 handleViewFile 为 undefined。
   * 调用方在此回调内实现页面跳转，hook 不依赖具体路由。
   */
  onViewFile?: (taskId: number) => void;
}

export function useUserAppExportAction({
  appId,
  exportType,
  params,
  onViewFile,
}: UseUserAppExportActionOptions): UseExportActionResult {
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const { success: showSuccess, error: showError } = useToast();
  const queryClient = useQueryClient();

  const shouldPollRef = useRef(false);

  const { data: activeCount = 0 } = useQuery({
    queryKey: ["exportAction", "user", appId, exportType, "active_count"],
    queryFn: async () => {
      const res = await userFileExportActiveCount({
        app_id: appId,
        export_type: exportType,
      });
      return res.response?.count ?? 0;
    },
    refetchInterval: () => (shouldPollRef.current ? 3000 : false),
    staleTime: 0,
  });

  useEffect(() => {
    if (activeCount === 0) {
      shouldPollRef.current = false;
    }
  }, [activeCount]);

  const { data: taskResult, isLoading } = useQuery({
    queryKey: ["exportAction", "user", appId, exportType, "tasks", currentPage],
    queryFn: async () => {
      const res = await userFileExportList({
        app_id: appId,
        export_type: exportType,
        page: { page: currentPage, limit: 20 },
      });
      const rawData = res.response?.data ?? [];

      const items: ExportTask[] = rawData.map((task) => ({
        id: task.task.id,
        status: task.task.status,
        createdAt: task.task.add_time || new Date(),
        completedAt: task.task.change_time ?? undefined,
        error: task.task.error_message,
        downloadPostUrl:
          task.task.status === 3
            ? "/api/user/app_export_task/download"
            : undefined,
        params: task.task.export_params,
        type: task.task.export_type,
      }));

      return { items, total: res.response?.total ?? 0 };
    },
    enabled: drawerOpen,
    refetchInterval: (query) => {
      if (!drawerOpen) return false;
      const list = query.state.data?.items ?? [];
      return list.some((t) => t.status === 1 || t.status === 2)
        ? 3000
        : false;
    },
  });

  const submitMutation = useMutation({
    mutationFn: async () => {
      await userFileExportSubmit({
        app_id: appId,
        export_type: exportType,
        params,
      });
    },
    onSuccess: () => {
      showSuccess("已提交导出任务");
      shouldPollRef.current = true;
      queryClient.invalidateQueries({
        queryKey: ["exportAction", "user", appId, exportType, "active_count"],
      });
      queryClient.invalidateQueries({
        queryKey: ["exportAction", "user", appId, exportType],
      });
      setDrawerOpen(true);
    },
    onError: (err: any) => {
      showError(err?.data?.message || err?.message || "提交导出任务失败");
    },
  });

  const tasks = taskResult?.items ?? [];
  const totalCount = taskResult?.total ?? 0;
  const totalPages = Math.ceil(totalCount / 20);

  const handleViewFile = onViewFile
    ? (taskId: number) => {
        setDrawerOpen(false);
        onViewFile(taskId);
      }
    : undefined;

  return {
    activeCount,
    isSubmitting: submitMutation.isPending,
    tasks,
    totalCount,
    currentPage,
    totalPages,
    drawerOpen,
    openDrawer: () => setDrawerOpen(true),
    closeDrawer: () => setDrawerOpen(false),
    submit: () => submitMutation.mutateAsync(),
    setPage: setCurrentPage,
    isLoadingTasks: isLoading,
    handleViewFile,
  };
}
