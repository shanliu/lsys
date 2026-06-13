/**
 * useAdminExportAction — 管理端导出操作 hook
 *
 * 职责：管理管理端导出相关的纯业务逻辑和状态：
 *   - 查询活跃任务数（带轮询）
 *   - 查询任务列表（抽屉打开时）
 *   - 提交导出任务
 *
 * 返回纯数据 + 操作函数，调用方负责渲染 UI。
 * 典型用法：
 *   const exp = useAdminExportAction({ exportType, params });
 *
 *   // 移动端触发按钮（FilterBar.MobileExtra）
 *   <ExportMobileButton activeCount={exp.activeCount} onClick={exp.openDrawer} />
 *
 *   // 桌面端导出按钮（FilterActions.extraActions）
 *   <ExportSplitButton onSubmitExport={exp.submit} onViewHistory={exp.openDrawer} ... />
 *
 *   // 移动端筛选 Drawer 底部（FilterBar.DrawerFooter）
 *   {(closeDrawer) => <Button onClick={() => { exp.submit(); closeDrawer(); }}>导出</Button>}
 *
 *   // 导出历史抽屉（页面根层级）
 *   <ExportDrawer open={exp.drawerOpen} onOpenChange={...} tasks={exp.tasks} ... />
 */

import { type ExportTask } from "@apps/main/components/export-manager";
import {
  adminExportActiveCount,
  adminExportList,
  adminExportSubmit,
} from "@shared/apis/admin/export";
import { useToast } from "@shared/contexts/toast-context";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

export interface UseAdminExportActionOptions {
  /** 导出类型常量 */
  exportType: string;
  params?: Record<string, unknown>;
}

export interface UseAdminExportActionResult {
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
}

export function useAdminExportAction({
  exportType,
  params,
}: UseAdminExportActionOptions): UseAdminExportActionResult {
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const { success: showSuccess, error: showError } = useToast();
  const queryClient = useQueryClient();

  // 轮询控制
  const shouldPollRef = useRef(false);

  // 查询活跃任务数
  const { data: activeCount = 0 } = useQuery({
    queryKey: ["exportAction", "system", exportType, "active_count"],
    queryFn: async () => {
      const res = await adminExportActiveCount({ export_type: exportType });
      return res.response?.count ?? 0;
    },
    refetchInterval: () => (shouldPollRef.current ? 3000 : false),
    staleTime: 0,
  });

  // 当 activeCount 变为 0 时，停止轮询
  useEffect(() => {
    if (activeCount === 0) {
      shouldPollRef.current = false;
    }
  }, [activeCount]);

  // 查询任务列表（当抽屉打开时）
  const { data: taskResult, isLoading } = useQuery({
    queryKey: ["exportAction", "system", exportType, "tasks", currentPage],
    queryFn: async () => {
      const res = await adminExportList({
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
            ? '/api/system/file/export_download'
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

  // 提交导出任务
  const submitMutation = useMutation({
    mutationFn: async () => {
      await adminExportSubmit({ export_type: exportType, params });
    },
    onSuccess: () => {
      showSuccess("已提交导出任务");
      shouldPollRef.current = true;
      queryClient.invalidateQueries({
        queryKey: ["exportAction", "system", exportType, "active_count"],
      });
      queryClient.invalidateQueries({
        queryKey: ["exportAction", "system", exportType],
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
