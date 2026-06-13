import {
  CursorPagination,
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  useLimitCountNum,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-file/list";
import {
  FileDownloadProgressInfoSchema,
  USER_APP_FILE_DOWNLOAD_PROGRESS_SSE_URL,
  userFileDelete,
  userFileDownloadingList,
  type FileDownloadProgressInfoType,
  type UserFileDownloadingItemType,
} from "@shared/apis/user/file";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { Button } from "@shared/components/ui/button";
import { Progress } from "@shared/components/ui/progress";
import { useToast } from "@shared/contexts/toast-context";
import { FilterTotalCount } from "@apps/main/components/filter-bar/filter-fields";
import { formatTotalCount } from "@shared/lib/utils/format-utils";
import { formatFileSize, formatTime, getQueryResponseCursor, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils";
import { Config } from "@shared/lib/config";
import { userStore } from "@shared/lib/auth";
import { useSse } from "@shared/hooks/use-sse";
import { type LimitType } from "@shared/types/base-schema";
import { type ColumnDef } from "@tanstack/react-table";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import {
  ArrowLeft,
  CheckCircle2,
  Download,
  ExternalLink,
  HardDrive,
  ListOrdered,
  Loader2,
  RefreshCw,
  Trash2,
  Wifi,
  WifiOff,
  XCircle,
} from "lucide-react";
import { type ReactNode, useCallback, useEffect, useMemo, useRef, useState } from "react";

// ──────────────────────────────────────────
// 工具函数
// ──────────────────────────────────────────

function formatSpeed(bps: number): string {
  if (bps <= 0) return "";
  const units = ["B/s", "KB/s", "MB/s", "GB/s"];
  const i = Math.min(Math.floor(Math.log(bps) / Math.log(1024)), units.length - 1);
  return `${(bps / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

function getProgressStatusBadge(status: string | undefined) {
  if (!status) return null;
  const lower = status.toLowerCase();
  if (lower === "in_progress" || lower === "downloading" || lower === "running") {
    return (
      <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
        <Loader2 className="h-3 w-3 animate-spin" />
        下载中
      </span>
    );
  }
  if (lower === "queued" || lower === "pending") {
    return (
      <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
        <ListOrdered className="h-3 w-3" />
        排队中
      </span>
    );
  }
  if (lower === "completed") {
    return (
      <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
        <CheckCircle2 className="h-3 w-3" />
        已完成
      </span>
    );
  }
  if (lower === "failed") {
    return (
      <span className="inline-flex items-center gap-1 text-xs text-muted-foreground/90">
        <XCircle className="h-3 w-3" />
        失败
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
      <ListOrdered className="h-3 w-3" />
      排队中
    </span>
  );
}

// ──────────────────────────────────────────
// 下载中文件表格
// ──────────────────────────────────────────

interface DownloadingTableProps {
  data: UserFileDownloadingItemType[];
  progressMap: Map<number, FileDownloadProgressInfoType>;
  loading?: boolean;
  error?: ReactNode;
  onDeleteFile: (id: number) => Promise<unknown>;
}

function DownloadingTable({ data, progressMap, loading, error, onDeleteFile }: DownloadingTableProps) {
  const columns: ColumnDef<UserFileDownloadingItemType>[] = useMemo(
    () => [
      {
        accessorKey: "id",
        header: "ID",
        size: 60,
        cell: ({ getValue }) => (
          <div className="py-1 text-xs text-muted-foreground">{getValue<number>()}</div>
        ),
      },
      {
        accessorKey: "file_name",
        header: "文件名 / 来源",
        cell: ({ row }) => {
          const file = row.original;
          const isLocal = file.storage_type?.startsWith("local");
          return (
            <div className="flex flex-col gap-0.5 py-1 max-w-[240px]">
              <div className="flex items-center gap-1">
                {isLocal ? (
                  <HardDrive className="h-3 w-3 flex-shrink-0 text-emerald-500" />
                ) : (
                  <Download className="h-3 w-3 flex-shrink-0 text-blue-400" />
                )}
                <span className="truncate text-sm font-medium" title={file.file_name}>
                  {file.file_name || "-"}
                </span>
              </div>
              {file.source_url && (
                <div className="flex items-center gap-1 pl-4">
                  <span
                    className="truncate text-xs text-muted-foreground leading-tight"
                    title={file.source_url}
                  >
                    {file.source_url}
                  </span>
                  <a
                    href={file.source_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex-shrink-0 text-muted-foreground/60 hover:text-muted-foreground"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <ExternalLink className="h-2.5 w-2.5" />
                  </a>
                </div>
              )}
            </div>
          );
        },
      },
      {
        accessorKey: "file_size",
        header: "文件大小",
        size: 100,
        cell: ({ getValue }) => (
          <div className="py-1 text-sm">{formatFileSize(getValue<number>())}</div>
        ),
      },
      {
        id: "progress",
        header: "下载进度",
        size: 240,
        cell: ({ row }) => {
          const file = row.original;
          const prog = progressMap.get(file.file_id);
          const chunkTotal = file.file_chunk_total ?? 0;
          const chunkSucc = file.file_chunk_succ ?? 0;
          const showChunkInfo = chunkTotal > 0;

          const statusBadge = file.is_downloading ? (
            <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
              <Loader2 className="h-3 w-3 animate-spin" />
              下载中
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
              <ListOrdered className="h-3 w-3" />
              排队中
            </span>
          );

          if (prog) {
            const pct = Math.min(Math.max(prog.percent, 0), 100);
            const speed = formatSpeed(prog.speed_bps);
            const isCompleted = prog.status === "completed";
            const displayDownloaded = isCompleted ? (prog.total_size || file.file_size) : prog.total_downloaded;
            const displayTotal = prog.total_size || file.file_size;
            return (
              <div className="flex flex-col gap-1 py-1 min-w-[190px]">
                <div className="flex items-center justify-between gap-2">
                  {getProgressStatusBadge(prog.status)}
                  <span className="flex items-center gap-1.5 text-xs text-muted-foreground tabular-nums">
                    <span>{pct.toFixed(1)}%</span>
                    {showChunkInfo && <span className="text-muted-foreground/60">{chunkSucc}/{chunkTotal} 片</span>}
                  </span>
                </div>
                <Progress value={pct} className="h-1.5" />
                <div className="flex items-center justify-between gap-2 text-xs text-muted-foreground whitespace-nowrap">
                  <span className="tabular-nums">
                    {formatFileSize(displayDownloaded)}&nbsp;/&nbsp;{formatFileSize(displayTotal)}
                  </span>
                  {speed && <span className="text-blue-500 tabular-nums">{speed}</span>}
                </div>
              </div>
            );
          }

          if (showChunkInfo) {
            const pct = (chunkSucc / chunkTotal) * 100;
            return (
              <div className="flex flex-col gap-1 py-1 min-w-[190px]">
                <div className="flex items-center justify-between gap-2">
                  {statusBadge}
                  <span className="text-xs text-muted-foreground tabular-nums">
                    {chunkSucc}/{chunkTotal} 片
                  </span>
                </div>
                <Progress value={pct} className="h-1.5" />
              </div>
            );
          }

          return (
            <div className="flex flex-col gap-1 py-1 min-w-[190px]">
              <div className="flex items-center gap-2">
                {statusBadge}
              </div>
            </div>
          );
        },
      },
      {
        accessorKey: "content_type",
        header: "内容类型",
        cell: ({ getValue }) => (
          <div
            className="max-w-[120px] truncate py-1 text-xs text-muted-foreground"
            title={getValue<string>() || ""}
          >
            {getValue<string>() || "-"}
          </div>
        ),
      },
      {
        accessorKey: "storage_type",
        header: "存储",
        size: 100,
        cell: ({ getValue }) => (
          <div className="py-1 text-xs text-muted-foreground truncate max-w-[90px]">
            {getValue<string>() || "-"}
          </div>
        ),
      },
      {
        accessorKey: "add_time",
        header: "添加时间",
        size: 120,
        cell: ({ getValue }) => {
          const addTime = getValue<Date | null>();
          return (
            <div className="text-xs py-1">
              {addTime ? formatTime(addTime, TIME_STYLE.RELATIVE_ELEMENT) : "-"}
            </div>
          );
        },
      },
      {
        id: "actions",
        header: () => <div className="text-center py-1">操作</div>,
        size: 80,
        cell: ({ row }) => {
          const file = row.original;
          return (
            <DataTableAction className="justify-end sm:justify-center">
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <ConfirmDialog
                  title="确认删除"
                  description={
                    <>
                      您确定要取消下载并删除文件{" "}
                      <strong>{file.file_name}</strong> 吗？
                    </>
                  }
                  onConfirm={async () => {
                    await onDeleteFile(file.id);
                  }}
                >
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-auto px-2 py-1 text-destructive hover:text-destructive"
                    title="取消下载"
                  >
                    <Trash2 className="h-3 w-3" />
                    <span className="text-xs ml-1">取消</span>
                  </Button>
                </ConfirmDialog>
              </DataTableActionItem>
            </DataTableAction>
          );
        },
      },
    ],
    [progressMap, onDeleteFile],
  );

  return (
    <DataTable
      data={data}
      columns={columns}
      loading={loading}
      error={error}
      scrollSnapDelay={300}
      leftStickyColumns={[{ column: 0, minWidth: "60px", maxWidth: "60px" }]}
      className="[&_tr]:h-auto [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
      tableContainerClassName="h-full"
    />
  );
}

// ──────────────────────────────────────────
// 下载中文件视图
// ──────────────────────────────────────────

interface FileListDownloadingViewProps {
  appId: number;
  onGoToRoot: () => void;
}

export function FileListDownloadingView({
  appId,
  onGoToRoot,
}: FileListDownloadingViewProps) {
  const queryClient = useQueryClient();
  const { error: showError } = useToast();
  const navigate = useNavigate();

  const filterParam = Route.useSearch();
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  const pagination: LimitType = {
    pos: filterParam.pos || null,
    limit: currentLimit,
    forward: filterParam.forward ?? true,
    more: true,
  };

  const searchGo = useSearchNavigate(navigate, filterParam);
  const countNumManager = useLimitCountNum({});

  // ──────────────────────────────────────────
  // 文件列表查询（HTTP，仅用于初始化和 SSE 结束后刷新）
  // ──────────────────────────────────────────
  const { data: downloadingData, isLoading, isError, error, isSuccess } = useQuery({
    queryKey: ["userFileDownloadingList", appId, pagination.pos, currentLimit, pagination.forward],
    queryFn: ({ signal }) =>
      userFileDownloadingList({ app_id: appId, is_downloading: undefined, limit: pagination, count_num: countNumManager.getCountNum() }, { signal }),
    staleTime: 0,
    gcTime: 0,
    // 每30秒轮询一次，同时由 SSE 驱动实时刷新
    refetchInterval: 30000,
  });

  isSuccess && countNumManager.handleQueryResult(downloadingData as any);

  const files: UserFileDownloadingItemType[] = useMemo(
    () => getQueryResponseData<UserFileDownloadingItemType[]>(downloadingData, []),
    [downloadingData],
  );
  const cursorData = getQueryResponseCursor(downloadingData);

  // ──────────────────────────────────────────
  // SSE 进度状态（key: file_id）
  // ──────────────────────────────────────────
  const [progressMap, setProgressMap] = useState<Map<number, FileDownloadProgressInfoType>>(
    () => new Map(),
  );

  // 记录上一次订阅的 file_id 集合（用于检测变化并重连）
  const prevIdsKeyRef = useRef<string>("");

  // 所有未完成文件的 ref_id 列表（用于 SSE 订阅）
  // 包含排队中和正在下载的文件，当下载开始时 Redis PUBLISH 会自动触发 SSE 推送
  const downloadingRefIds = useMemo(
    () => files.map((f) => f.id),
    [files],
  );

  const sseBody = useMemo(
    () =>
      downloadingRefIds.length > 0
        ? JSON.stringify({ ref_ids: downloadingRefIds })
        : null,
    [downloadingRefIds],
  );

  const sseEnabled = downloadingRefIds.length > 0;

  const handleSseMessage = useCallback((raw: string) => {
    try {
      const parsed = JSON.parse(raw);
      console.log("[sse] raw message:", parsed);
      const result = FileDownloadProgressInfoSchema.safeParse(parsed);
      if (!result.success) {
        console.error("[sse] schema parse failed:", result.error.issues, "raw:", parsed);
        return;
      }
      const info = result.data;
      console.log("[sse] setting progressMap file_id=", info.file_id, "pct=", info.percent);
      setProgressMap((prev) => new Map(prev).set(info.file_id, info));
    } catch (e) {
      console.error("[sse] JSON parse error:", e, "raw:", raw);
    }
  }, []);

  // SSE 流正常结束（所有订阅文件完成）→ 刷新列表
  const handleSseComplete = useCallback(() => {
    setProgressMap(new Map());
    queryClient.invalidateQueries({ queryKey: ["userFileDownloadingList"] });
    queryClient.invalidateQueries({ queryKey: ["userFileList"] });
  }, [queryClient]);

  const { status: sseStatus, disconnect: disconnectSse } = useSse({
    url: `${Config.apiBaseUrl}${USER_APP_FILE_DOWNLOAD_PROGRESS_SSE_URL}`,
    method: "POST",
    body: sseBody,
    getHeaders: () => {
      const loginData = userStore.getState().current();
      const headers: Record<string, string> = {};
      if (loginData) headers["Authorization"] = `Bearer ${loginData.bearer}`;
      return headers;
    },
    onMessage: handleSseMessage,
    onComplete: handleSseComplete,
    enabled: sseEnabled,
  });

  const handleGoToRoot = useCallback(() => {
    disconnectSse();
    setProgressMap(new Map());
    onGoToRoot();
  }, [disconnectSse, onGoToRoot]);

  // 当下载文件 ref_id 列表变化时，清空旧进度数据
  // 无需手动 reconnect：useSse 在 body（sseBody）变化时会自动重连
  useEffect(() => {
    const idsKey = [...downloadingRefIds].sort().join(",");
    if (idsKey !== prevIdsKeyRef.current) {
      prevIdsKeyRef.current = idsKey;
      setProgressMap(new Map());
    }
  }, [downloadingRefIds]);

  // ──────────────────────────────────────────
  // 删除文件
  // ──────────────────────────────────────────
  const deleteFileMutation = useMutation({
    mutationFn: (id: number) => userFileDelete({ file_ref_id: id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["userFileDownloadingList"] });
      queryClient.invalidateQueries({ queryKey: ["userFileList"] });
    },
    onError: (err: any) => {
      showError(err?.data?.message || err?.message || "删除文件失败");
    },
  });

  const refreshData = () =>
    queryClient.refetchQueries({ queryKey: ["userFileDownloadingList"] });

  // SSE 状态指示器
  const sseIndicator = sseEnabled ? (
    sseStatus === "connected" ? (
      <span className="flex items-center gap-1 text-xs text-emerald-600">
        <Wifi className="h-3 w-3" />
        实时
      </span>
    ) : sseStatus === "connecting" ? (
      <span className="flex items-center gap-1 text-xs text-muted-foreground">
        <Wifi className="h-3 w-3 animate-pulse" />
        连接中
      </span>
    ) : (
      <span className="flex items-center gap-1 text-xs text-amber-500">
        <WifiOff className="h-3 w-3" />
        已断开
      </span>
    )
  ) : null;

  return (
    <div className="flex flex-col min-h-0 space-y-3">
      {/* 顶部导航 + 状态栏 */}
      <div className="flex-shrink-0">
        <div className="bg-card rounded-lg border shadow-sm px-4 py-3 flex items-center gap-3">
          <Button size="sm" variant="ghost" onClick={handleGoToRoot} className="-ml-1 flex-shrink-0">
            <ArrowLeft className="h-4 w-4 mr-1" />
            返回
          </Button>
          <div className="flex items-center gap-2 flex-1 min-w-0">
            <span className="text-sm font-medium">下载中的文件</span>
            {sseIndicator}
          </div>
          <FilterTotalCount value={formatTotalCount(countNumManager.getTotalInfo())} loading={isLoading} />
          <Button
            size="sm"
            variant="ghost"
            onClick={refreshData}
            disabled={isLoading}
            className="flex-shrink-0"
          >
            <RefreshCw className="h-4 w-4" />
            <span className="hidden sm:inline ml-1">刷新</span>
          </Button>
        </div>
      </div>

      <div className="flex-1 flex flex-col min-h-0">
        <div className="flex-1 overflow-hidden">
          <DownloadingTable
            data={files}
            progressMap={progressMap}
            loading={isLoading}
            error={
              isError ? (
                <CenteredError error={error} variant="content" onReset={refreshData} />
              ) : undefined
            }
            onDeleteFile={(id) => deleteFileMutation.mutateAsync(id)}
          />
        </div>
        <div className="flex-shrink-0 pt-4 pb-4">
          {countNumManager.hasTotalInfo() && (
            <CursorPagination
              limit={currentLimit}
              cursorData={cursorData}
              searchGo={searchGo}
              totalInfo={countNumManager.getTotalInfo()}
              currentPageSize={files.length}
              loading={isLoading}
              onRefresh={refreshData}
              showRefresh
              showPageSize
              pageSizeOptions={PAGE_SIZE_OPTIONS}
              onPageSizeChange={(pageSize) =>
                searchGo({ limit: pageSize, pos: null, forward: true })
              }
            />
          )}
        </div>
      </div>
    </div>
  );
}

