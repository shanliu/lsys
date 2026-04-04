/**
 * AdminExportAction - 系统管理端导出面板组件
 *
 * 负责管理导出相关的业务逻辑：
 * - 查询活跃任务数
 * - 查询任务列表
 * - 提交导出任务
 * - 轮询控制
 *
 * 将数据传递给纯 UI 组件 ExportAction 进行渲染。
 */

import {
  ExportAction,
  ExportTask,
} from "@apps/main/components/filter-container/export-action";
import {
  adminExportActiveCount,
  adminExportList,
  adminExportSubmit,
} from "@shared/apis/admin/export";
import { useToast } from "@shared/contexts/toast-context";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import React, { useEffect, useRef, useState } from "react";

// ─── 状态码转换 ────────────────────────────────────────────────────────────

/**
 * 将后端状态码转换为 UI 友好的状态字符串
 * 1=Pending 2=Running 3=Success 4=Failed 5=Deleted
 */
function convertStatus(code: number): ExportTask["status"] {
  const map: Record<number, ExportTask["status"]> = {
    1: "pending",
    2: "running",
    3: "success",
    4: "failed",
    5: "deleted",
  };
  return map[code] ?? "pending";
}

// ─── 系统管理端导出面板组件 ────────────────────────────────────────────────────

export function AdminExportAction({
  exportType,
  params,
  drawerTitle,
  layoutParams,
}: {
  /** 导出类型常量 */
  exportType: string;
  params?: Record<string, unknown>;
  drawerTitle?: React.ReactNode;
  layoutParams?: { isMobile: boolean };
}) {
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
        page: { page: currentPage, limit: 20 },
      });
      const rawData = res.response?.data ?? [];

      // 转换成 UI 需要的格式
      const items: ExportTask[] = rawData.map((task) => ({
        id: task.id,
        status: convertStatus(task.status),
        createdAt: task.add_time || new Date(),
        completedAt: task.change_time ?? undefined,
        error: task.error_message,
        file: task.file
          ? {
              name: task.file.file_name,
              url: task.file.file_url ?? "",
              size: task.file.file_size,
            }
          : undefined,
        params: task.export_params,
        type: task.export_type,
      }));

      return {
        items,
        total: res.response?.total ?? 0,
      };
    },
    enabled: drawerOpen,
    refetchInterval: (query) => {
      if (!drawerOpen) return false;
      const result = query.state.data;
      const list = result?.items ?? [];
      return list.some((t) => t.status === "pending" || t.status === "running")
        ? 3000
        : false;
    },
  });

  // 提交导出任务
  const submitMutation = useMutation({
    mutationFn: async () => {
      await adminExportSubmit({
        export_type: exportType,
        params,
      });
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

  return (
    <ExportAction
      activeCount={activeCount}
      onButtonClick={() => setDrawerOpen(true)}
      drawerOpen={drawerOpen}
      onDrawerOpenChange={setDrawerOpen}
      tasks={tasks}
      totalCount={totalCount}
      currentPage={currentPage}
      totalPages={totalPages}
      onPageChange={setCurrentPage}
      isLoading={isLoading}
      onSubmitExport={() => submitMutation.mutate()}
      isSubmitting={submitMutation.isPending}
      drawerTitle={drawerTitle}
      layoutParams={layoutParams}
    />
  );
}
