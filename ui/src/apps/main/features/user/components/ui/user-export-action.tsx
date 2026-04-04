/**
 * UserExportAction - 用户端导出面板组件
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
  userFileExportActiveCount,
  userFileExportList,
  userFileExportSubmit,
} from "@shared/apis/user/file";
import { useToast } from "@shared/contexts/toast-context";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
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

// ─── 用户端导出面板组件 ────────────────────────────────────────────────────

export function UserExportAction({
  appId,
  exportType,
  params,
  drawerTitle,
  enableViewFile = true,
  layoutParams,
}: {
  /** 应用 ID（账号级导出时可省略） */
  appId?: number;
  /** 导出类型常量 */
  exportType: string;
  params?: Record<string, unknown>;
  drawerTitle?: React.ReactNode;
  /**
   * 是否启用"查看文件列表"跳转功能（需要 appId，默认 true）
   * 点击后跳转至 /user/app/:appId/features-file/list，并以 tag_name 过滤
   */
  enableViewFile?: boolean;
  layoutParams?: { isMobile: boolean };
}) {
  const navigate = useNavigate();
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const { success: showSuccess, error: showError } = useToast();
  const queryClient = useQueryClient();

  // 轮询控制
  const shouldPollRef = useRef(false);

  // 查询活跃任务数
  const { data: activeCount = 0 } = useQuery({
    queryKey: [
      "exportAction",
      "user",
      appId ?? "global",
      exportType,
      "active_count",
    ],
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

  // 当 activeCount 变为 0 时，停止轮询
  useEffect(() => {
    if (activeCount === 0) {
      shouldPollRef.current = false;
    }
  }, [activeCount]);

  // 查询任务列表（当抽屉打开时）
  const { data: taskResult, isLoading } = useQuery({
    queryKey: [
      "exportAction",
      "user",
      appId ?? "global",
      exportType,
      "tasks",
      currentPage,
    ],
    queryFn: async () => {
      const res = await userFileExportList({
        app_id: appId,
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
      await userFileExportSubmit({
        app_id: appId as number,
        export_type: exportType,
        params,
      });
    },
    onSuccess: () => {
      showSuccess("已提交导出任务");
      shouldPollRef.current = true;
      queryClient.invalidateQueries({
        queryKey: [
          "exportAction",
          "user",
          appId ?? "global",
          exportType,
          "active_count",
        ],
      });
      queryClient.invalidateQueries({
        queryKey: ["exportAction", "user", appId ?? "global", exportType],
      });
      setDrawerOpen(true);
    },
    onError: (err: any) => {
      showError(err?.data?.message || err?.message || "提交导出任务失败");
    },
  });

  // 点击"查看文件列表"：先关闭抽屉，再跳转
  const handleViewFile =
    enableViewFile && appId != null
      ? (taskId: number) => {
          setDrawerOpen(false);
          navigate({
            to: "/user/app/$appId/features-file/list",
            params: { appId: appId as any },
            search: { tag_name: `export_${taskId}` } as any,
          });
        }
      : undefined;

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
      onViewFile={handleViewFile}
      onSubmitExport={() => submitMutation.mutate()}
      isSubmitting={submitMutation.isPending}
      drawerTitle={drawerTitle}
      layoutParams={layoutParams}
    />
  );
}
