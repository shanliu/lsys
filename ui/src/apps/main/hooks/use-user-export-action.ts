/**
 * useUserExportAction — 账号级导出操作 hook（无 appId）
 *
 * 适用于账号维度的全局导出（如登录日志、账号文件列表等），
 * 不依赖任何 appId，使用 /api/user/export_task/* 系列接口。
 *
 * 返回纯数据 + 操作函数，调用方负责渲染 UI。
 * 示例：
 *   const exp = useUserExportAction({ exportType, params });
 *   <ExportMobileButton activeCount={exp.activeCount} onClick={exp.openDrawer} />
 *   <ExportSplitButton onSubmitExport={exp.submit} onViewHistory={exp.openDrawer} ... />
 *   <ExportDrawer open={exp.drawerOpen} ... tasks={exp.tasks} ... />
 */

import { type ExportTask } from "@apps/main/components/export-manager";
import {
  userExportActiveCount,
  userExportList,
  userExportSubmit,
} from "@shared/apis/user/file";
import { useToast } from "@shared/contexts/toast-context";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

export interface UseUserExportActionOptions {
  /** 导出类型常量 */
  exportType: string;
  params?: Record<string, unknown>;
}

/** 导出操作 hook 的通用返回类型（user / user-app 共享） */
export interface UseExportActionResult {
  /** 活跃任务数（角标数量） */
  activeCount: number;
  /** 是否正在提交导出任务 */
  isSubmitting: boolean;
  /** 导出任务列表 */
  tasks: ExportTask[];
  /** 任务总数 */
  totalCount: number;
  /** 当前页码 */
  currentPage: number;
  /** 总页数 */
  totalPages: number;
  /** 导出历史抽屉是否打开 */
  drawerOpen: boolean;
  /** 打开导出历史抽屉 */
  openDrawer: () => void;
  /** 关闭导出历史抽屉 */
  closeDrawer: () => void;
  /** 提交导出任务，返回 Promise，成功 resolve，失败 reject（错误已由 hook 内部 toast 处理） */
  submit: () => Promise<void>;
  /** 切换任务列表页码 */
  setPage: (page: number) => void;
  /** 任务列表是否正在加载 */
  isLoadingTasks: boolean;
  /** 点击"查看文件列表"回调（仅 useUserAppExportAction 可能有值） */
  handleViewFile?: (taskId: number) => void;
}

export function useUserExportAction({
  exportType,
  params,
}: UseUserExportActionOptions): UseExportActionResult {
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const { success: showSuccess, error: showError } = useToast();
  const queryClient = useQueryClient();

  const shouldPollRef = useRef(false);

  const { data: activeCount = 0 } = useQuery({
    queryKey: ["exportAction", "user", "global", exportType, "active_count"],
    queryFn: async () => {
      const res = await userExportActiveCount({ export_type: exportType });
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
    queryKey: ["exportAction", "user", "global", exportType, "tasks", currentPage],
    queryFn: async () => {
      const res = await userExportList({
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
          task.task.status === 3 ? "/api/user/export_task/download" : undefined,
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
      await userExportSubmit({ export_type: exportType, params });
    },
    onSuccess: () => {
      showSuccess("已提交导出任务");
      shouldPollRef.current = true;
      queryClient.invalidateQueries({
        queryKey: ["exportAction", "user", "global", exportType, "active_count"],
      });
      queryClient.invalidateQueries({
        queryKey: ["exportAction", "user", "global", exportType],
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
  };
}
