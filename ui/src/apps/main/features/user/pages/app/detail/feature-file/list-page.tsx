import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterContentSearch } from "@apps/main/components/filter-container/filter-content-search";
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select";
import { FilterTagCombobox } from "@apps/main/components/filter-container/filter-tag-combobox";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import {
  useDictData,
  type TypedDictData,
} from "@apps/main/hooks/use-dict-data";
import {
  CursorPagination,
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  useLimitCountNum,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-file/list";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  userFileDelete,
  userFileList,
  userFileTagNames,
  EXPORT_TYPE_USER_FILE_LIST,
  type UserFileItemType,
} from "@shared/apis/user/file";
import { UserExportAction } from "@apps/main/features/user/components/ui/user-export-action";
import { DataTable } from "@shared/components/custom//table";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import {
  DataTableAction,
  DataTableActionItem,
} from "@shared/components/custom/table";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@shared/components/ui/tooltip";
import { useToast } from "@shared/contexts/toast-context";
import {
  cn,
  formatFileSize,
  formatTime,
  getQueryResponseCursor,
  getQueryResponseData,
  TIME_STYLE,
} from "@shared/lib/utils";
import { formatTotalCount } from "@shared/lib/utils/format-utils";
import { type LimitType } from "@shared/types/base-schema";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import {
  Cloud,
  Columns,
  Download,
  Eye,
  FileText,
  Link,
  Tags,
  Trash2,
  Upload,
} from "lucide-react";
import React, { useState } from "react";
import { featureFileModuleConfig } from "../nav-info";
import { FileChunksDrawer } from "./file-chunks-drawer";
import { FileDetailDrawer } from "./file-detail-drawer";
import { FileLogsDrawer } from "./file-logs-drawer";
import { FileTagsDrawer } from "./file-tags-drawer";
import { FileUploadDialog } from "./file-upload-dialog";
import { FileUrlDownloadDialog } from "./file-url-download-dialog";
import { CONTENT_SEARCH_TYPES, FileListFilterFormSchema } from "./list-schema";

export default function AppDetailFeatureFileListPage() {
  const { appId } = Route.useParams();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["user_file"] as const);

  if (dictError && dictErrors.length > 0) {
    return (
      <CenteredError variant="page" error={dictErrors} onReset={refetchDict} />
    );
  }

  if (dictIsLoading) {
    return <PageSkeletonTable variant="page" />;
  }

  const onUploadSuccess = () => {
    // 重置分页：先重置查询缓存，然后清除分页参数
    queryClient.invalidateQueries({ queryKey: ["userFileList"] });
    // 使用 setTimeout 确保在对话框关闭后重置分页
    setTimeout(() => {
      navigate({
        to: "/user/app/$appId/features-file/list",
        params: { appId },
        search: {},
        replace: true,
      });
    }, 100);
  };

  const onDownloadSuccess = () => {
    // 重置分页：先重置查询缓存，然后清除分页参数
    queryClient.invalidateQueries({ queryKey: ["userFileList"] });
    // 使用 setTimeout 确保在对话框关闭后重置分页
    setTimeout(() => {
      navigate({
        to: "/user/app/$appId/features-file/list",
        params: { appId },
        search: {},
        replace: true,
      });
    }, 100);
  };

  return (
    <>
      <AppDetailNavContainer
        {...featureFileModuleConfig}
        actions={
          <div className="flex flex-wrap items-center gap-2">
            <FileUploadDialog
              appId={Number(appId)}
              uploadConfig={dictData}
              onSuccess={onUploadSuccess}
            >
              <Button size="sm" variant="default">
                <Upload className="h-4 w-4 mr-1" />
                上传文件
              </Button>
            </FileUploadDialog>
            <FileUrlDownloadDialog
              appId={Number(appId)}
              onSuccess={onDownloadSuccess}
            >
              <Button size="sm" variant="outline">
                <Link className="h-4 w-4 mr-1" />
                URL 下载
              </Button>
            </FileUrlDownloadDialog>
          </div>
        }
      >
        <AppDetailFeatureFileListContent
          appId={Number(appId)}
          dictData={dictData}
        />
      </AppDetailNavContainer>
    </>
  );
}

interface AppDetailFeatureFileListContentProps {
  appId: number;
  dictData: TypedDictData<["user_file"]>;
}

function AppDetailFeatureFileListContent({
  appId,
  dictData,
}: AppDetailFeatureFileListContentProps) {
  const queryClient = useQueryClient();
  const { success: showSuccess, error: showError } = useToast();
  const navigate = useNavigate();

  const filterParam = Route.useSearch();
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 详细信息抽屉状态
  const [detailDrawerOpen, setDetailDrawerOpen] = useState(false);
  const [detailFile, setDetailFile] = useState<UserFileItemType | null>(null);

  // 日志抽屉状态
  const [logsDrawerOpen, setLogsDrawerOpen] = useState(false);
  const [logsFile, setLogsFile] = useState<UserFileItemType | null>(null);

  // 分片抽屉状态
  const [chunksDrawerOpen, setChunksDrawerOpen] = useState(false);
  const [chunksFile, setChunksFile] = useState<UserFileItemType | null>(null);

  // 标签抽屉状态
  const [tagsDrawerOpen, setTagsDrawerOpen] = useState(false);
  const [tagsFile, setTagsFile] = useState<UserFileItemType | null>(null);

  // 过滤条件
  const filters = {
    status: filterParam.status || null,
    tag_name: filterParam.tag_name || null,
    content_type: filterParam.content_type || null,
    content_value: filterParam.content_value || null,
  };

  // 分页
  const pagination: LimitType = {
    pos: filterParam.pos || null,
    limit: filterParam.limit || DEFAULT_PAGE_SIZE,
    forward: filterParam.forward ?? true,
    more: true,
  };

  const searchGo = useSearchNavigate(navigate, filterParam);
  const countNumManager = useLimitCountNum(filters);

  // 根据 content_type 映射到 API 参数
  const contentFilterParams = React.useMemo(() => {
    if (!filters.content_type || !filters.content_value) return {};
    const map: Record<string, string> = {
      file_md5: "file_md5",
      source_url: "source_url",
      url: "url",
    };
    const key = map[filters.content_type];
    return key ? { [key]: filters.content_value } : {};
  }, [filters.content_type, filters.content_value]);

  // 获取文件列表
  const {
    data: fileData,
    isSuccess,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: [
      "userFileList",
      appId,
      pagination.pos,
      currentLimit,
      pagination.forward,
      pagination.more,
      filters.status,
      filters.tag_name,
      filters.content_type,
      filters.content_value,
    ],
    queryFn: ({ signal }) =>
      userFileList(
        {
          app_id: Number(appId),
          limit: pagination,
          count_num: countNumManager.getCountNum(),
          status: filters.status ?? undefined,
          tag_names: filters.tag_name ? [filters.tag_name] : undefined,
          ...contentFilterParams,
        },
        { signal },
      ),
    placeholderData: (previousData) => previousData,
  });

  isSuccess && countNumManager.handleQueryResult(fileData);

  const files = getQueryResponseData<UserFileItemType[]>(fileData, []);
  const cursorData = getQueryResponseCursor(fileData);

  // 删除文件
  const deleteFileMutation = useMutation({
    mutationFn: (params: { file_user_id: number }) =>
      userFileDelete({ file_user_id: params.file_user_id }),
    onSuccess: () => {
      showSuccess("文件已删除");
      queryClient.invalidateQueries({ queryKey: ["userFileList"] });
    },
    onError: (error: any) => {
      showError(error?.data?.message || error?.message || "删除文件失败");
    },
  });

  const handleDeleteFile = async (fileUserid: number) => {
    await deleteFileMutation.mutateAsync({ file_user_id: fileUserid });
  };

  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["userFileList"] });
  };

  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["userFileList"] });
  };

  // 状态映射 — 基于 FileStatus 枚举
  // 1=Normal, 2=Deleted, 3=Unfinished, 4=Failed
  const fileStatus = createStatusMapper(
    {
      1: "success", // Normal   - 正常
      2: "danger", // Deleted  - 已删除
      3: "info", // Unfinished - 未完成（上传中）
      4: "danger", // Failed   - 失败
    },
    (status) => dictData.file_status?.getLabel(String(status), "") ?? "",
  );

  // 表格列定义
  const columns: ColumnDef<UserFileItemType>[] = [
    {
      accessorKey: "id",
      header: "ID",
      size: 60,
      cell: ({ getValue }) => (
        <div className="py-1 text-xs text-muted-foreground">
          {getValue<number>()}
        </div>
      ),
    },
    {
      accessorKey: "file_name",
      header: "文件名",
      cell: ({ row }) => {
        const fileName = row.original.file_name;
        const storageType = row.original.storage_type;
        const isCloud = storageType && storageType !== "local";
        return (
          <div className="flex items-center gap-1 py-1 max-w-[220px]">
            <span className="truncate text-sm" title={fileName}>
              {fileName || "-"}
            </span>
            {isCloud && (
              <TooltipProvider delayDuration={200}>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Cloud className="h-3.5 w-3.5 flex-shrink-0 text-blue-400 cursor-pointer" />
                  </TooltipTrigger>
                  <TooltipContent side="top">
                    <span>{storageType}</span>
                  </TooltipContent>
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
      size: 100,
      cell: ({ getValue }) => (
        <div className="py-1 text-sm">{formatFileSize(getValue<number>())}</div>
      ),
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
      accessorKey: "tag_count",
      header: "标签",
      size: 140,
      cell: ({ row }) => {
        const file = row.original;
        const tagCount = file.tag_count ?? 0;
        const firstTag = file.first_tag;
        // 如果有完整 tags 数据（attr_tag=true 时），使用 tags
        const tags = file.tags;
        const displayFirstTag =
          firstTag ?? (tags && tags.length > 0 ? tags[0] : null);
        const displayCount = tagCount > 0 ? tagCount : (tags?.length ?? 0);

        if (displayCount === 0) {
          return (
            <div
              className="flex items-center gap-1 py-1 cursor-pointer group"
              onClick={() => {
                setTagsFile(file);
                setTagsDrawerOpen(true);
              }}
              title="点击添加标签"
            >
              <span className="text-xs text-muted-foreground group-hover:text-foreground transition-colors">
                -
              </span>
              <Tags className="h-3 w-3 text-muted-foreground/50 group-hover:text-muted-foreground transition-colors flex-shrink-0 ml-0.5" />
            </div>
          );
        }

        return (
          <div
            className="flex items-center gap-1 py-1 cursor-pointer group"
            onClick={() => {
              setTagsFile(file);
              setTagsDrawerOpen(true);
            }}
            title="点击管理标签"
          >
            {displayFirstTag && (
              <Badge
                variant="secondary"
                className="text-xs px-1.5 py-0 truncate max-w-[90px] group-hover:bg-secondary/60 transition-colors"
                title={displayFirstTag.tag_name}
              >
                {displayFirstTag.tag_name}
              </Badge>
            )}
            {displayCount > 1 && (
              <Badge
                variant="outline"
                className="text-xs px-1.5 py-0 flex-shrink-0 group-hover:bg-accent transition-colors"
              >
                +{displayCount - 1}
              </Badge>
            )}
            <Tags className="h-3 w-3 text-muted-foreground/50 group-hover:text-muted-foreground transition-colors flex-shrink-0 ml-0.5" />
          </div>
        );
      },
    },
    {
      accessorKey: "status",
      header: "状态",
      size: 80,
      cell: ({ getValue }) => {
        const status = getValue<number>();
        return (
          <div className="py-1">
            <Badge className={cn(fileStatus.getClass(status))}>
              {fileStatus.getText(status)}
            </Badge>
          </div>
        );
      },
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
      size: 100,
      cell: ({ row }) => {
        const file = row.original;
        return (
          <DataTableAction className="justify-end sm:justify-center">
            {/* 详细信息 */}
            <DataTableActionItem
              mobileDisplay="display"
              desktopDisplay="collapsed"
            >
              <Button
                variant="ghost"
                size="sm"
                className={cn("h-auto px-2 py-1")}
                title="详细信息"
                onClick={() => {
                  setDetailFile(file);
                  setDetailDrawerOpen(true);
                }}
              >
                <Eye className="h-3 w-3" />
                <span className="text-xs ml-1">详细</span>
              </Button>
            </DataTableActionItem>

            {/* 日志 */}
            <DataTableActionItem
              mobileDisplay="display"
              desktopDisplay="collapsed"
            >
              <Button
                variant="ghost"
                size="sm"
                className={cn("h-auto px-2 py-1")}
                title="操作日志"
                onClick={() => {
                  setLogsFile(file);
                  setLogsDrawerOpen(true);
                }}
              >
                <FileText className="h-3 w-3" />
                <span className="text-xs ml-1">日志</span>
              </Button>
            </DataTableActionItem>

            {/* 分片 */}
            {file.file_chunk_total && file.file_chunk_total > 1 ? (
              <DataTableActionItem
                mobileDisplay="display"
                desktopDisplay="collapsed"
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn("h-auto px-2 py-1")}
                  title="文件分片"
                  onClick={() => {
                    setChunksFile(file);
                    setChunksDrawerOpen(true);
                  }}
                >
                  <Columns className="h-3 w-3" />
                  <span className="text-xs ml-1">分片</span>
                </Button>
              </DataTableActionItem>
            ) : null}

            {/* 下载 */}
            {file.url && file.status === 1 ? (
              <DataTableActionItem
                mobileDisplay="display"
                desktopDisplay="collapsed"
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn("h-auto px-2 py-1")}
                  title="下载文件"
                  onClick={() => {
                    if (file.url) {
                      window.open(file.url, "_blank");
                    }
                  }}
                >
                  <Download className="h-3 w-3" />
                  <span className="text-xs ml-1">下载</span>
                </Button>
              </DataTableActionItem>
            ) : null}

            {/* 删除 */}
            <DataTableActionItem
              mobileDisplay="display"
              desktopDisplay="collapsed"
            >
              <ConfirmDialog
                title="确认删除"
                description={
                  <>
                    您确定要删除文件 <strong>{file.file_name}</strong>{" "}
                    吗？删除后将无法恢复。
                  </>
                }
                onConfirm={async () => await handleDeleteFile(file.id)}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn(
                    "h-auto px-2 py-1 text-destructive hover:text-destructive",
                  )}
                  title="删除文件"
                >
                  <Trash2 className="h-3 w-3" />
                  <span className="text-xs ml-1">删除</span>
                </Button>
              </ConfirmDialog>
            </DataTableActionItem>
          </DataTableAction>
        );
      },
    },
  ];

  return (
    <>
      <div className="flex flex-col min-h-0 space-y-3">
        <div className="flex-shrink-0 mb-1 sm:mb-4 space-y-3">
          {/* 过滤器 */}
          <FilterContainer
            defaultValues={{
              status: filterParam.status?.toString(),
              tag_name: filterParam.tag_name,
              content_type: filterParam.content_type,
              content_value: filterParam.content_value,
            }}
            resolver={zodResolver(FileListFilterFormSchema) as any}
            onSubmit={(data) => {
              const transformedData = data as {
                status?: number;
                tag_name?: string;
                content_type?: string;
                content_value?: string;
              };
              searchGo({
                status: transformedData.status,
                tag_name: transformedData.tag_name,
                content_type: transformedData.content_type,
                content_value: transformedData.content_type
                  ? transformedData.content_value
                  : undefined,
                pos: null,
                forward: true,
              });
            }}
            onReset={() => {
              searchGo({
                pos: null,
                limit: currentLimit,
                forward: true,
                status: undefined,
                tag_name: undefined,
                content_type: undefined,
                content_value: undefined,
              });
            }}
            countComponent={
              <FilterTotalCount
                value={formatTotalCount(countNumManager.getTotalInfo())}
                loading={isLoading}
              />
            }
            className="bg-card rounded-lg border shadow-sm relative"
          >
            {(layoutParams, form) => (
              <div className="flex-1 flex flex-wrap items-end gap-3">
                {/* 状态过滤 */}
                {dictData.file_status && (
                  <FilterDictSelect
                    name="status"
                    placeholder="选择状态"
                    label="状态"
                    disabled={isLoading}
                    dictData={dictData.file_status}
                    layoutParams={layoutParams}
                    allLabel="全部"
                    className={
                      layoutParams.isMobile
                        ? undefined
                        : "min-w-[100px] max-w-[130px]"
                    }
                  />
                )}

                {/* 标签过滤 */}
                <FilterTagCombobox
                  name="tag_name"
                  placeholder="选择标签"
                  searchPlaceholder="搜索标签..."
                  label="标签"
                  disabled={isLoading}
                  allLabel="全部"
                  layoutParams={layoutParams}
                  fetchTagNames={async (prefix: string) => {
                    const res = await queryClient.fetchQuery({
                      queryKey: ["userFileTagNames", appId, prefix],
                      queryFn: () =>
                        userFileTagNames({
                          app_id: Number(appId),
                          tag_name_prefix: prefix || undefined,
                          limit: 5,
                        }),
                      staleTime: 30_000,
                    });
                    return res?.response?.data ?? [];
                  }}
                />

                {/* 文件内容过滤 */}
                <FilterContentSearch
                  typeName="content_type"
                  valueName="content_value"
                  options={
                    CONTENT_SEARCH_TYPES as unknown as {
                      value: string;
                      label: string;
                    }[]
                  }
                  label="文件内容"
                  typePlaceholder="选择类型"
                  valuePlaceholder={(type) => {
                    const placeholders: Record<string, string> = {
                      file_md5: "输入文件MD5",
                      source_url: "输入来源URL",
                      url: "输入本地URL",
                    };
                    return placeholders[type] || "请输入...";
                  }}
                  disabled={isLoading}
                  layoutParams={layoutParams}
                />

                {/* 动作按钮区域 */}
                <div
                  className={cn(
                    layoutParams.isMobile ? "w-full" : "flex-shrink-0",
                  )}
                >
                  <FilterActions
                    form={form}
                    loading={isLoading}
                    layoutParams={layoutParams}
                    onRefreshSearch={clearCacheAndReload}
                    extraActions={
                      <UserExportAction
                        appId={appId}
                        exportType={EXPORT_TYPE_USER_FILE_LIST}
                        params={{
                          status: filters.status ?? undefined,
                          tag_names: filters.tag_name
                            ? [filters.tag_name]
                            : undefined,
                          ...contentFilterParams,
                        }}
                        layoutParams={layoutParams}
                      />
                    }
                  />
                </div>
              </div>
            )}
          </FilterContainer>
        </div>

        {/* 表格和分页容器 */}
        <div className="flex-1 flex flex-col min-h-0">
          <div className="flex-1 overflow-hidden">
            <DataTable
              data={files}
              columns={columns}
              loading={isLoading}
              error={
                isError ? (
                  <CenteredError
                    error={error}
                    variant="content"
                    onReset={refreshData}
                  />
                ) : null
              }
              scrollSnapDelay={300}
              leftStickyColumns={[
                { column: 0, minWidth: "60px", maxWidth: "60px" },
              ]}
              className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
              tableContainerClassName="h-full"
            />
          </div>

          {/* 分页控件 */}
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
                showRefresh={true}
                showPageSize={true}
                pageSizeOptions={PAGE_SIZE_OPTIONS}
                onPageSizeChange={(pageSize) => {
                  searchGo({
                    limit: pageSize,
                    pos: null,
                    forward: true,
                  });
                }}
              />
            )}
          </div>
        </div>

        {/* 详细信息抽屉 */}
        {detailFile && (
          <FileDetailDrawer
            open={detailDrawerOpen}
            onOpenChange={setDetailDrawerOpen}
            file={detailFile}
            dictData={dictData}
          />
        )}

        {/* 日志抽屉 */}
        {logsFile && (
          <FileLogsDrawer
            appId={Number(appId)}
            file={logsFile}
            isOpen={logsDrawerOpen}
            onOpenChange={setLogsDrawerOpen}
          />
        )}

        {/* 分片抽屉 */}
        {chunksFile && (
          <FileChunksDrawer
            appId={Number(appId)}
            file={chunksFile}
            isOpen={chunksDrawerOpen}
            onOpenChange={setChunksDrawerOpen}
            dictData={dictData}
          />
        )}

        {/* 标签抽屉 */}
        {tagsFile && (
          <FileTagsDrawer
            file={tagsFile}
            isOpen={tagsDrawerOpen}
            onOpenChange={setTagsDrawerOpen}
            onTagsChanged={() => {
              queryClient.invalidateQueries({ queryKey: ["userFileList"] });
            }}
          />
        )}
      </div>
    </>
  );
}
