import { type TypedDictData } from "@apps/main/hooks/use-dict-data";
import {
  CursorPagination,
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  useLimitCountNum,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-file/list";
import {
  userFileDelete,
  userFileLineageRelatedList,
  type UserFileItemType,
} from "@shared/apis/user/file";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import { formatFileSize, formatTotalCount } from "@shared/lib/utils/format-utils";
import { getQueryResponseCursor, getQueryResponseData } from "@shared/lib/utils";
import { type LimitType } from "@shared/types/base-schema";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ArrowLeft, GitBranch, RefreshCw } from "lucide-react";
import { useMemo } from "react";
import { FileListTable } from "./file-list-table";

interface FileListLineageViewProps {
  appId: number;
  dictData: TypedDictData<["user_file", "user_export"]>;
  sourceFileId: number;
  sourceFile: UserFileItemType | null;
  relTypeFilter?: number | null;
  onSwitchToLineageView: (file: UserFileItemType, relType?: number | null) => void;
  onRelTypeChange: (relType?: number | null) => void;
  onGoBack: () => void;
  onGoToRoot: () => void;
}

export function FileListLineageView({
  appId,
  dictData,
  sourceFileId,
  sourceFile,
  relTypeFilter,
  onSwitchToLineageView,
  onRelTypeChange,
  onGoBack,
  onGoToRoot,
}: FileListLineageViewProps) {
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
  const countNumManager = useLimitCountNum({ sourceFileId, relTypeFilter });

  const { data: lineageData, isLoading, isError, error, isSuccess } = useQuery({
    queryKey: ["userFileLineageRelatedList", appId, sourceFileId, relTypeFilter, pagination.pos, currentLimit, pagination.forward],
    queryFn: ({ signal }) =>
      userFileLineageRelatedList({ id: sourceFileId, rel_type: relTypeFilter ?? null, limit: pagination, count_num: false }, { signal }),
    placeholderData: (prev) => prev,
  });

  isSuccess && countNumManager.handleQueryResult(lineageData as any);

  const files = useMemo(() => getQueryResponseData<UserFileItemType[]>(lineageData, []), [lineageData]);
  const cursorData = getQueryResponseCursor(lineageData);

  const deleteFileMutation = useMutation({
    mutationFn: (id: number) => userFileDelete({ file_ref_id: id }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["userFileLineageRelatedList"] }),
    onError: (err: any) => { showError(err?.data?.message || err?.message || "删除文件失败"); },
  });

  const refreshData = () => queryClient.refetchQueries({ queryKey: ["userFileLineageRelatedList"] });
  const goToDownloadingView = () => {
    navigate({
      to: "/user/app/$appId/features-file/list",
      params: { appId },
      search: { mode: "downloading" },
    });
  };

  const getStorageTypeName = (storageType: string) =>
    dictData?.storage_type?.find(s => s.key === storageType)?.val || storageType;

  const hasRelTypeFilter = sourceFile?.lineage_counts && sourceFile.lineage_counts.length > 0;

  return (
    <div className="flex flex-col min-h-0 space-y-3">
      {/* 统一卡片：导航路径 + 来源文件信息 + 关联类型筛选 */}
      <div className="flex-shrink-0">
        <div className="bg-card rounded-lg border shadow-sm p-4 space-y-3">
          {/* 导航路径行 */}
          <div className="flex items-center gap-2">
            <Button size="sm" variant="ghost" onClick={onGoBack} className="-ml-1 flex-shrink-0">
              <ArrowLeft className="h-4 w-4 mr-1" />
              返回
            </Button>
            <div className="flex items-center gap-1 text-sm overflow-x-auto flex-1 min-w-0">
              <button
                onClick={onGoToRoot}
                className="text-muted-foreground hover:text-foreground hover:underline cursor-pointer whitespace-nowrap"
              >
                文件列表
              </button>
              <span className="text-muted-foreground px-1">/</span>
              <span
                className="font-medium text-foreground truncate"
                title={sourceFile?.file_name ?? `文件 #${sourceFileId}`}
              >
                {sourceFile?.file_name ?? `文件 #${sourceFileId}`}
              </span>
              <GitBranch className="h-3.5 w-3.5 text-primary flex-shrink-0 ml-1" />
            </div>
            <span className="text-sm text-muted-foreground flex-shrink-0">
              {formatTotalCount(countNumManager.getTotalInfo())}
            </span>
          </div>

          {/* 来源文件元信息（有 sourceFile 时展示） */}
          {sourceFile && (
            <div className="flex items-center gap-2 flex-wrap text-xs text-muted-foreground border-t pt-2">
              <Badge variant="outline" className="text-xs px-1.5 py-0">ID: {sourceFileId}</Badge>
              <span>{formatFileSize(sourceFile.file_size)}</span>
              {sourceFile.content_type && <span className="opacity-70">{sourceFile.content_type}</span>}
              {sourceFile.storage_type && <span className="opacity-70">{getStorageTypeName(sourceFile.storage_type)}</span>}
            </div>
          )}

          {/* 关联类型筛选 + 刷新（有关联类型数据时才展示） */}
          {hasRelTypeFilter && (
            <div className="flex items-center justify-between flex-wrap gap-3 border-t pt-2">
              <div className="flex flex-wrap gap-2">
                <Button
                  size="sm"
                  variant={relTypeFilter == null ? "default" : "outline"}
                  onClick={() => onRelTypeChange(undefined)}
                >
                  全部
                </Button>
                {sourceFile!.lineage_counts!.map(c => (
                  <Button
                    key={c.rel_type}
                    size="sm"
                    variant={relTypeFilter === c.rel_type ? "default" : "outline"}
                    onClick={() => onRelTypeChange(c.rel_type)}
                  >
                    {dictData.lineage_rel_type?.getLabel(String(c.rel_type), `类型${c.rel_type}`)}
                    <span className="ml-1 opacity-60">({c.count})</span>
                  </Button>
                ))}
              </div>
              <Button size="sm" variant="ghost" onClick={refreshData} disabled={isLoading}>
                <RefreshCw className="h-4 w-4" />
              </Button>
            </div>
          )}

          {/* 无关联类型数据时只展示刷新 */}
          {!hasRelTypeFilter && (
            <div className="flex justify-end border-t pt-2">
              <Button size="sm" variant="ghost" onClick={refreshData} disabled={isLoading}>
                <RefreshCw className="h-4 w-4" />
              </Button>
            </div>
          )}
        </div>
      </div>

      <div className="flex-1 flex flex-col min-h-0">
        <div className="flex-1 overflow-hidden">
          <FileListTable
            appId={appId} dictData={dictData} data={files} loading={isLoading}
            error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : undefined}
            onGoToDownloadingPage={goToDownloadingView}
            onSwitchToLineageView={onSwitchToLineageView}
            onDeleteFile={(id) => deleteFileMutation.mutateAsync(id)}
            onTagsChanged={() => queryClient.invalidateQueries({ queryKey: ["userFileLineageRelatedList"] })}
          />
        </div>
        <div className="flex-shrink-0 pt-4 pb-4">
          {countNumManager.hasTotalInfo() && (
            <CursorPagination limit={currentLimit} cursorData={cursorData} searchGo={searchGo}
              totalInfo={countNumManager.getTotalInfo()} currentPageSize={files.length} loading={isLoading}
              onRefresh={refreshData} showRefresh showPageSize pageSizeOptions={PAGE_SIZE_OPTIONS}
              onPageSizeChange={(pageSize) => searchGo({ limit: pageSize, pos: null, forward: true })} />
          )}
        </div>
      </div>
    </div>
  );
}
