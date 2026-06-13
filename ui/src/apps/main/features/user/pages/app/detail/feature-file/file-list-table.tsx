import { PostDownload } from "@apps/main/components/local/post-download";
import { FilePublicUrlDialog } from "@apps/main/features/user/components/ui/file-public-url-dialog";
import { type TypedDictData } from "@apps/main/hooks/use-dict-data";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@shared/components/ui/tooltip";
import { type UserFileItemType } from "@shared/apis/user/file";
import { getAppFileShareUrl } from "@shared/lib/apis/api_read";
import { cn, formatFileSize, formatTime, TIME_STYLE } from "@shared/lib/utils";
import { type ColumnDef } from "@tanstack/react-table";
import {
  ArrowDown,
  Clock,
  Cloud,
  Columns,
  Copy,
  Download,
  Eye,
  FileText,
  GitBranch,
  HardDrive,
  Lock,
  Share2,
  ShieldCheck,
  Tags,
  Trash2,
} from "lucide-react";
import { type ReactNode, useState, useMemo } from "react";
import { FileCopyDialog } from "./file-copy-dialog";
import { FileChunksDrawer } from "./file-chunks-drawer";
import { FileDetailDrawer } from "./file-detail-drawer";
import { FileExpireTimeDialog } from "./file-expire-time-dialog";
import { FileLogsDrawer } from "./file-logs-drawer";
import { FileTagsDrawer } from "./file-tags-drawer";

export interface FileListTableProps {
  appId: number;
  dictData: TypedDictData<["user_file", "user_export"]>;
  data: UserFileItemType[];
  loading?: boolean;
  error?: ReactNode;
  onGoToDownloadingPage?: () => void;
  onSwitchToLineageView: (file: UserFileItemType, relType?: number | null) => void;
  onDeleteFile: (id: number) => Promise<unknown>;
  /** 标签修改后刷新数据 */
  onTagsChanged?: () => void;
}

export function FileListTable({
  appId,
  dictData,
  data,
  loading,
  error,
  onGoToDownloadingPage,
  onSwitchToLineageView,
  onDeleteFile,
  onTagsChanged,
}: FileListTableProps) {
  // 存储类型工具（从 dictData 中计算）
  const storageTypes = useMemo(() => dictData?.storage_type || [], [dictData?.storage_type]);

  const storageTypePrivateMap = useMemo(() => {
    const map = new Map<string, boolean>();
    storageTypes.forEach(st => {
      map.set(st.key, st.key.startsWith("local_") ? st.key !== "local_public" : (st.is_private ?? false));
    });
    return map;
  }, [storageTypes]);

  const storageTypeNameMap = useMemo(() => {
    const map = new Map<string, string>();
    storageTypes.forEach(st => map.set(st.key, st.val));
    return map;
  }, [storageTypes]);

  const isPublicStorage = (storageType: string) => storageTypePrivateMap.get(storageType) === false;
  const getStorageTypeName = (storageType: string) => storageTypeNameMap.get(storageType) || storageType;

  // 抽屉/对话框状态
  const [detailDrawerOpen, setDetailDrawerOpen] = useState(false);
  const [detailFile, setDetailFile] = useState<UserFileItemType | null>(null);
  const [logsDrawerOpen, setLogsDrawerOpen] = useState(false);
  const [logsFile, setLogsFile] = useState<UserFileItemType | null>(null);
  const [chunksDrawerOpen, setChunksDrawerOpen] = useState(false);
  const [chunksFile, setChunksFile] = useState<UserFileItemType | null>(null);
  const [tagsDrawerOpen, setTagsDrawerOpen] = useState(false);
  const [tagsFile, setTagsFile] = useState<UserFileItemType | null>(null);
  const [copyDialogOpen, setCopyDialogOpen] = useState(false);
  const [copyFile, setCopyFile] = useState<UserFileItemType | null>(null);
  const [expireDialogOpen, setExpireDialogOpen] = useState(false);
  const [expireFile, setExpireFile] = useState<UserFileItemType | null>(null);

  const fileStatus = createStatusMapper(
    { 1: "success", 2: "danger", 3: "info", 4: "danger" },
    (status) => dictData.file_status.getLabel(String(status), ""),
  );

  const columns: ColumnDef<UserFileItemType>[] = useMemo(
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
        header: "文件名",
        cell: ({ row }) => {
          const fileName = row.original.file_name;
          const storageType = row.original.storage_type;
          const isLocal = storageType?.startsWith("local");
          const isCloud = !!storageType && !isLocal;
          const isCrypto = storageType === "local_crypto";
          const isPrivate = !isPublicStorage(storageType);
          const storageName = getStorageTypeName(storageType);
          return (
            <div className="flex items-center gap-1 py-1 max-w-[220px]">
              <span className="truncate text-sm" title={fileName}>{fileName || "-"}</span>
              {isLocal && (
                <TooltipProvider delayDuration={200}>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <HardDrive className="h-3 w-3 flex-shrink-0 text-emerald-500 cursor-pointer" />
                    </TooltipTrigger>
                    <TooltipContent side="top"><span>{storageName}</span></TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )}
              {isCloud && (
                <TooltipProvider delayDuration={200}>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Cloud className="h-3 w-3 flex-shrink-0 text-blue-400 cursor-pointer" />
                    </TooltipTrigger>
                    <TooltipContent side="top"><span>{storageName}</span></TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )}
              {isCrypto && (
                <TooltipProvider delayDuration={200}>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <ShieldCheck className="h-3 w-3 flex-shrink-0 text-amber-500/70 cursor-pointer" />
                    </TooltipTrigger>
                    <TooltipContent side="top"><span>加密存储</span></TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )}
              {isPrivate && (
                <TooltipProvider delayDuration={200}>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Lock className="h-3 w-3 flex-shrink-0 text-muted-foreground/50 cursor-pointer" />
                    </TooltipTrigger>
                    <TooltipContent side="top"><span>私有文件</span></TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )}
            </div>
          );
        },
      },
      {
        accessorKey: "file_size",
        header: "文件大小",
        size: 110,
        minSize: 90,
        cell: ({ row, getValue }) => {
          const isDownloading = !!row.original.is_downloading;
          return (
            <div className="py-1 text-sm flex items-center gap-1.5 whitespace-nowrap">
              <span>{formatFileSize(getValue<number>())}</span>
              {isDownloading && (
                <TooltipProvider delayDuration={200}>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <button
                        type="button"
                        className="inline-flex h-3 w-3 flex-shrink-0 items-center justify-center rounded-full border border-blue-400/60 text-blue-500 hover:border-blue-500 hover:text-blue-600 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                        title="下载中，点击查看下载列表"
                        onClick={(e) => {
                          e.stopPropagation();
                          onGoToDownloadingPage?.();
                        }}
                      >
                        <span className="relative block h-1.5 w-1.5 overflow-hidden">
                          <ArrowDown className="absolute inset-0 h-1.5 w-1.5 animate-arrow-down-loop1" />
                          <ArrowDown className="absolute inset-0 h-1.5 w-1.5 animate-arrow-down-loop2" />
                        </span>
                      </button>
                    </TooltipTrigger>
                    <TooltipContent side="top"><span>下载中，点击查看</span></TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )}
            </div>
          );
        },
      },
      {
        accessorKey: "content_type",
        header: "内容类型",
        cell: ({ getValue }) => (
          <div className="max-w-[120px] truncate py-1 text-xs text-muted-foreground" title={getValue<string>() || ""}>
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
        id: "file_attrs",
        header: "状态&属性",
        size: 165,
        cell: ({ row }) => {
          const file = row.original;
          if (file.status !== 1) {
            const statusColorMap: Record<number, string> = {
              2: "text-red-500/60",
              3: "text-blue-400/70",
              4: "text-red-400/70",
            };
            const statusText = fileStatus.getText(file.status);
            return (
              <div className="py-1 overflow-hidden max-w-full">
                <Badge
                  variant="outline"
                  className={cn("text-xs px-1.5 py-0 font-normal max-w-full truncate", statusColorMap[file.status] ?? "text-muted-foreground")}
                  title={statusText}
                >
                  {statusText}
                </Badge>
              </div>
            );
          }
          const tagCount = file.tag_count ?? 0;
          const tags = file.tags;
          const firstTag = file.first_tag ?? (tags && tags.length > 0 ? tags[0] : null);
          const displayCount = tagCount > 0 ? tagCount : (tags?.length ?? 0);
          const counts = file.lineage_counts;
          const totalCount = counts?.reduce((sum, c) => sum + c.count, 0) ?? 0;
          return (
            <div className="flex items-center gap-1.5 py-1 overflow-hidden">
              <div className="flex items-center gap-1 cursor-pointer group"
                onClick={() => { setTagsFile(file); setTagsDrawerOpen(true); }}
                title={displayCount === 0 ? "点击添加标签" : "点击管理标签"}>
                {displayCount === 0 ? (
                  <span className="inline-flex items-center justify-center text-xs text-muted-foreground/50 group-hover:text-muted-foreground transition-colors flex-shrink-0">
                    <Tags className="h-3 w-3" />
                  </span>
                ) : (
                  <>
                    {firstTag && (
                      <Badge variant="secondary"
                        className="text-xs px-1.5 py-0 truncate max-w-[60px] group-hover:bg-secondary/60 transition-colors"
                        title={firstTag.tag_name}>
                        {firstTag.tag_name}
                      </Badge>
                    )}
                    <Badge variant="outline"
                      className="text-xs px-1 py-0 gap-0.5 flex-shrink-0 group-hover:bg-accent transition-colors font-normal">
                      <Tags className="h-3 w-3" />
                      {displayCount}
                    </Badge>
                  </>
                )}
              </div>
              {totalCount > 0 && (
                <TooltipProvider delayDuration={200}>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <div className="flex items-center gap-1 cursor-pointer group"
                        onClick={() => onSwitchToLineageView(file)}>
                        <Badge variant="outline"
                          className="text-xs px-1 py-0 gap-0.5 flex-shrink-0 group-hover:bg-accent transition-colors font-normal">
                          <GitBranch className="h-3 w-3" />
                          {totalCount}
                        </Badge>
                      </div>
                    </TooltipTrigger>
                    <TooltipContent side="right" className="max-w-xs bg-popover text-popover-foreground border shadow-md">
                      <div className="text-xs space-y-2">
                        <div className="font-medium border-b pb-1">文件关联详情</div>
                        <div className="space-y-1.5">
                          {counts?.map((c, i) => (
                            <div key={i} className="flex items-center justify-between gap-3">
                              <div className="flex items-center gap-2">
                                <Badge variant="secondary" className="text-xs px-1.5 py-0">
                                  {dictData.lineage_rel_type?.getLabel(String(c.rel_type), `类型${c.rel_type}`)}
                                </Badge>
                                <span className="text-muted-foreground">{getStorageTypeName(c.storage_type)}</span>
                              </div>
                              <span className="font-medium">{c.count} 个</span>
                            </div>
                          ))}
                        </div>
                        <div className="text-muted-foreground pt-2 border-t">共 {totalCount} 个关联文件</div>
                      </div>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )}
            </div>
          );
        },
      },
      {
        id: "actions",
        header: () => <div className="text-center py-1">操作</div>,
        size: 100,
        cell: ({ row }) => {
          const file = row.original;
          return (
            <DataTableAction className="justify-end sm:justify-center">
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <Button variant="ghost" size="sm" className="h-auto px-2 py-1" title="详细信息"
                  onClick={() => { setDetailFile(file); setDetailDrawerOpen(true); }}>
                  <Eye className="h-3 w-3" /><span className="text-xs ml-1">详细</span>
                </Button>
              </DataTableActionItem>
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <Button variant="ghost" size="sm" className="h-auto px-2 py-1" title="操作日志"
                  onClick={() => { setLogsFile(file); setLogsDrawerOpen(true); }}>
                  <FileText className="h-3 w-3" /><span className="text-xs ml-1">日志</span>
                </Button>
              </DataTableActionItem>
              {file.file_chunk_total && file.file_chunk_total > 1 ? (
                <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                  <Button variant="ghost" size="sm" className="h-auto px-2 py-1" title="文件分片"
                    onClick={() => { setChunksFile(file); setChunksDrawerOpen(true); }}>
                    <Columns className="h-3 w-3" /><span className="text-xs ml-1">分片</span>
                  </Button>
                </DataTableActionItem>
              ) : null}
              {file.file_key && file.status === 1 ? (
                <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                  <PostDownload url="/api/user/app_file/read" body={{ key: file.file_key }}>
                    {({ onClick, isLoading }) => (
                      <Button variant="ghost" size="sm" className="h-auto px-2 py-1" title="下载文件"
                        onClick={onClick} disabled={isLoading}>
                        <Download className="h-3 w-3" />
                        <span className="text-xs ml-1">{isLoading ? "下载中..." : "下载"}</span>
                      </Button>
                    )}
                  </PostDownload>
                </DataTableActionItem>
              ) : null}
              {file.file_key && file.status === 1 && isPublicStorage(file.storage_type) ? (
                <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                  <FilePublicUrlDialog url={getAppFileShareUrl(file.file_key!)}>
                    <Button variant="ghost" size="sm" className="h-auto px-2 py-1" title="公开链接">
                      <Share2 className="h-3 w-3" /><span className="sm:hidden text-xs ml-1">链接</span><span className="hidden sm:inline text-xs ml-1">公开链接</span>
                    </Button>
                  </FilePublicUrlDialog>
                </DataTableActionItem>
              ) : null}
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <Button variant="ghost" size="sm" className="h-auto px-2 py-1" title="拷贝文件"
                  onClick={() => { setCopyFile(file); setCopyDialogOpen(true); }}>
                  <Copy className="h-3 w-3" /><span className="text-xs ml-1">拷贝</span>
                </Button>
              </DataTableActionItem>
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <Button variant="ghost" size="sm" className="h-auto px-2 py-1" title="更新过期时间"
                  onClick={() => { setExpireFile(file); setExpireDialogOpen(true); }}>
                  <Clock className="h-3 w-3" /><span className="sm:hidden text-xs ml-1">过期</span><span className="hidden sm:inline text-xs ml-1">过期时间</span>
                </Button>
              </DataTableActionItem>
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <ConfirmDialog
                  title="确认删除"
                  description={<>您确定要删除文件 <strong>{file.file_name}</strong> 吗？删除后将无法恢复。</>}
                  onConfirm={async () => { await onDeleteFile(file.id); }}
                >
                  <Button variant="ghost" size="sm"
                    className="h-auto px-2 py-1 text-destructive hover:text-destructive" title="删除文件">
                    <Trash2 className="h-3 w-3" /><span className="text-xs ml-1">删除</span>
                  </Button>
                </ConfirmDialog>
              </DataTableActionItem>
            </DataTableAction>
          );
        },
      },
    ],
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [dictData, storageTypePrivateMap, storageTypeNameMap, onSwitchToLineageView, onDeleteFile],
  );

  return (
    <>
      <DataTable
        data={data}
        columns={columns}
        loading={loading}
        error={error}
        scrollSnapDelay={300}
        leftStickyColumns={[{ column: 0, minWidth: "60px", maxWidth: "60px" }]}
        className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
        tableContainerClassName="h-full"
      />

      {detailFile && (
        <FileDetailDrawer open={detailDrawerOpen} onOpenChange={setDetailDrawerOpen} file={detailFile} dictData={dictData} />
      )}
      {logsFile && (
        <FileLogsDrawer appId={appId} file={logsFile} isOpen={logsDrawerOpen} onOpenChange={setLogsDrawerOpen} />
      )}
      {chunksFile && (
        <FileChunksDrawer appId={appId} file={chunksFile} isOpen={chunksDrawerOpen} onOpenChange={setChunksDrawerOpen} dictData={dictData} />
      )}
      {tagsFile && (
        <FileTagsDrawer file={tagsFile} isOpen={tagsDrawerOpen} onOpenChange={setTagsDrawerOpen} onTagsChanged={onTagsChanged} />
      )}
      {copyFile && (
        <FileCopyDialog open={copyDialogOpen} onOpenChange={setCopyDialogOpen} appId={appId} file={copyFile} storageTypes={storageTypes} />
      )}
      {expireFile && (
        <FileExpireTimeDialog open={expireDialogOpen} onOpenChange={setExpireDialogOpen} appId={appId} file={expireFile} />
      )}
    </>
  );
}
