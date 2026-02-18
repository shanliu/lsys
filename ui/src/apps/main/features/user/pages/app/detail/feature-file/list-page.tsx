import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterDictSelect } from "@apps/main/components/filter-container/filter-dict-select";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import {
    DEFAULT_PAGE_SIZE,
    OffsetPagination,
    PAGE_SIZE_OPTIONS,
    useCountNumManager,
    useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-file/list";
import { zodResolver } from "@hookform/resolvers/zod";
import {
    userFileDelete,
    userFileList,
    type UserFileItemType,
} from "@shared/apis/user/file";
import { DataTable } from "@shared/components/custom//table";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import {
    cn,
    formatTime,
    getQueryResponseCursor,
    getQueryResponseData,
    TIME_STYLE,
} from "@shared/lib/utils";
import { type LimitType } from "@shared/types/base-schema";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import { Download, Eye, FileText, Link, Trash2, Upload } from "lucide-react";
import { useState } from "react";
import { featureFileModuleConfig } from "../nav-info";
import { FileDetailDrawer } from "./file-detail-drawer";
import { FileLogsDrawer } from "./file-logs-drawer";
import { FileUploadDialog } from "./file-upload-dialog";
import { FileUrlDownloadDialog } from "./file-url-download-dialog";
import { FileListFilterFormSchema } from "./list-schema";

export default function AppDetailFeatureFileListPage() {
    const { appId } = Route.useParams();
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
        queryClient.invalidateQueries({ queryKey: ["userFileList"] });
    };

    return (
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
                        uploadConfig={dictData}
                        onSuccess={onUploadSuccess}
                    >
                        <Button size="sm" variant="outline">
                            <Link className="h-4 w-4 mr-1" />
                            URL 下载
                        </Button>
                    </FileUrlDownloadDialog>
                </div>
            }
        >
            <AppDetailFeatureFileListContent dictData={dictData} />
        </AppDetailNavContainer>
    );
}

interface AppDetailFeatureFileListContentProps {
    dictData: TypedDictData<["user_file"]>;
}

function AppDetailFeatureFileListContent({ dictData }: AppDetailFeatureFileListContentProps) {
    const { appId } = Route.useParams();
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

    // 过滤条件
    const filters = {
        storage_type: filterParam.storage_type || null,
        file_md5: filterParam.file_md5 || null,
        status: filterParam.status || null,
    };

    // 分页
    const pagination: LimitType = {
        pos: filterParam.pos || null,
        limit: filterParam.limit || DEFAULT_PAGE_SIZE,
        forward: filterParam.forward ?? true,
        more: true,
    };

    const searchGo = useSearchNavigate(navigate, filterParam);
    const countNumManager = useCountNumManager(filters);

    // 获取文件列表
    const { data: fileData, isSuccess, isLoading, isError, error } = useQuery({
        queryKey: [
            "userFileList",
            appId,
            pagination.pos,
            currentLimit,
            pagination.forward,
            pagination.more,
            filters.storage_type,
            filters.file_md5,
            filters.status,
        ],
        queryFn: ({ signal }) =>
            userFileList(
                {
                    app_id: Number(appId),
                    limit: pagination,
                    count_num: countNumManager.getCountNum(),
                    storage_type: filters.storage_type,
                    file_md5: filters.file_md5,
                },
                { signal }
            ),
        placeholderData: (previousData) => previousData,
    });

    isSuccess && countNumManager.handleLimitQueryResult(fileData);

    const files = getQueryResponseData<UserFileItemType[]>(fileData, []);
    const cursorData = getQueryResponseCursor(fileData);

    // 删除文件
    const deleteFileMutation = useMutation({
        mutationFn: (params: { file_id: number }) =>
            userFileDelete({ app_id: Number(appId), file_id: params.file_id }),
        onSuccess: () => {
            showSuccess("文件已删除");
            queryClient.invalidateQueries({ queryKey: ["userFileList"] });
        },
        onError: (error: any) => {
            showError(error?.data?.message || error?.message || "删除文件失败");
        },
    });

    const handleDeleteFile = async (fileId: number) => {
        await deleteFileMutation.mutateAsync({ file_id: fileId });
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
            1: "success",   // Normal   - 正常
            2: "danger",    // Deleted  - 已删除
            3: "info",      // Unfinished - 未完成（上传中）
            4: "danger",    // Failed   - 失败
        },
        (status) =>
            dictData.file_status?.getLabel(String(status)) || String(status),
    );

    const formatFileSize = (size: number): string => {
        if (size === 0) return "0 B";
        const units = ["B", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(size) / Math.log(1024));
        return parseFloat((size / Math.pow(1024, i)).toFixed(2)) + " " + units[i];
    };

    // 表格列定义
    const columns: ColumnDef<UserFileItemType>[] = [
        {
            accessorKey: "file_user_id",
            header: "ID",
            size: 60,
            cell: ({ getValue }) => (
                <div className="py-1 text-xs text-muted-foreground">{getValue<number>()}</div>
            ),
        },
        {
            accessorKey: "file_name",
            header: "文件名",
            cell: ({ getValue }) => (
                <div className="max-w-[200px] truncate py-1 text-sm" title={getValue<string>()}>
                    {getValue<string>() || "-"}
                </div>
            ),
        },
        {
            accessorKey: "storage_type",
            header: "存储类型",
            size: 100,
            cell: ({ getValue }) => (
                <div className="py-1 text-sm">{getValue<string>() || "-"}</div>
            ),
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
                <div className="max-w-[120px] truncate py-1 text-xs text-muted-foreground" title={getValue<string>() || ""}>
                    {getValue<string>() || "-"}
                </div>
            ),
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
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
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
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
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

                        {/* 下载 */}
                        {file.url && file.status === 1 ? (
                            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
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
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <ConfirmDialog
                                title="确认删除"
                                description={
                                    <>
                                        您确定要删除文件 <strong>{file.file_name}</strong> 吗？删除后将无法恢复。
                                    </>
                                }
                                onConfirm={async () => await handleDeleteFile(file.id)}
                            >
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    className={cn("h-auto px-2 py-1 text-destructive hover:text-destructive")}
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
                            storage_type: filterParam.storage_type,
                            file_md5: filterParam.file_md5,
                            status: filterParam.status?.toString(),
                        }}
                        resolver={zodResolver(FileListFilterFormSchema) as any}
                        onSubmit={(data) => {
                            const transformedData = data as {
                                storage_type?: string;
                                file_md5?: string;
                                status?: number;
                            };
                            searchGo({
                                storage_type: transformedData.storage_type,
                                file_md5: transformedData.file_md5,
                                status: transformedData.status,
                                pos: null,
                                forward: true,
                            });
                        }}
                        onReset={() => {
                            searchGo({
                                pos: null,
                                limit: currentLimit,
                                forward: true,
                                storage_type: undefined,
                                file_md5: undefined,
                                status: undefined,
                            });
                        }}
                        countComponent={
                            <FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />
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
                                    />
                                )}

                                {/* 存储类型过滤 */}
                                <FilterInput
                                    name="storage_type"
                                    placeholder="输入存储类型"
                                    label="存储类型"
                                    disabled={isLoading}
                                    layoutParams={layoutParams}
                                />

                                {/* MD5过滤 */}
                                <FilterInput
                                    name="file_md5"
                                    placeholder="输入文件MD5"
                                    label="文件MD5"
                                    disabled={isLoading}
                                    layoutParams={layoutParams}
                                />

                                {/* 动作按钮区域 */}
                                <div className={cn(layoutParams.isMobile ? "w-full" : "flex-shrink-0")}>
                                    <FilterActions
                                        form={form}
                                        loading={isLoading}
                                        layoutParams={layoutParams}
                                        onRefreshSearch={clearCacheAndReload}
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
                            error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}
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
                        {(countNumManager.getTotal() ?? 0) > 0 && (
                            <OffsetPagination
                                limit={currentLimit}
                                cursorData={cursorData}
                                searchGo={searchGo}
                                total={countNumManager.getTotal()}
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
            </div>
        </>
    );
}
