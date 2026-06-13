import { FilterBar } from "@apps/main/components/filter-bar/container";
import { FilterActions } from "@apps/main/components/filter-bar/filter-actions/filter-actions";
import { FilterSearchButton } from "@apps/main/components/filter-bar/filter-actions/filter-search-button";
import { FilterResetButton } from "@apps/main/components/filter-bar/filter-actions/filter-reset-button";
import { FilterDictSelect, FilterTotalCount } from "@apps/main/components/filter-bar/filter-fields";
import { useFilterBarForm } from "@apps/main/hooks/use-filter-bar-form";
import { ExportButton, ExportMobileButton, ExportSplitButton } from "@apps/main/components/export-manager/export-buttons";
import { ExportDrawer } from "@apps/main/components/export-manager/export-drawer";
import { useUserAppExportAction } from "@apps/main/hooks/use-user-app-export-action";
import { EXPORT_TYPE_APP_SCRIPT_RECORDS } from "@shared/apis/user/file";
import { formatTotalCount } from "@shared/lib/utils/format-utils";
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import {
    useDictData,
    type TypedDictData,
} from "@apps/main/hooks/use-dict-data";
import {
    DEFAULT_PAGE_SIZE,
    PagePagination,
    usePageCountNum,
} from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-file/collector";
import { zodResolver } from "@hookform/resolvers/zod";
import {
    userCollectorScriptDelete,
    userCollectorScriptList,
    userCollectorScriptStatus,
    type CollectorScriptItemType,
} from "@shared/apis/user/collector";
import { DataTable } from "@shared/components/custom//table";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import {
    cn,
    formatServerError,
    formatTime,
    getQueryResponseData,
    TIME_STYLE,
} from "@shared/lib/utils";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import { Eye, FileText, FolderOpen, Play, Plus, Power, PowerOff, ScrollText, Trash2 } from "lucide-react";
import { useState } from "react";
import { featureFileModuleConfig } from "../nav-info";
import { CollectorFilesDrawer } from "./collector-files-drawer";
import { CollectorListFilterFormSchema } from "./collector-list-schema";
import * as z from "zod";
import { CollectorLogsDrawer } from "./collector-logs-drawer";
import { CollectorRecordListDrawer } from "./collector-record-list-drawer";
import { CollectorScriptDrawer } from "./collector-script-drawer";
import { CollectorTriggerDialog } from "./collector-trigger-dialog";

export default function AppDetailFeatureCollectorListPage() {
    const { appId } = Route.useParams();
    const queryClient = useQueryClient();

    // 字典数据
    const {
        dictData,
    } = useDictData(["user_collector", "user_export"] as const);

    // 新增/编辑脚本抽屉
    const [scriptDrawerOpen, setScriptDrawerOpen] = useState(false);
    const [editScript, setEditScript] = useState<CollectorScriptItemType | null>(null);

    const onScriptSuccess = () => {
        queryClient.invalidateQueries({ queryKey: ["collectorScriptList"] });
        setScriptDrawerOpen(false);
        setEditScript(null);
    };

    return (
        <AppDetailNavContainer
            {...featureFileModuleConfig}
            actions={
                <Button
                    size="sm"
                    variant="default"
                    onClick={() => {
                        setEditScript(null);
                        setScriptDrawerOpen(true);
                    }}
                >
                    <Plus className="h-4 w-4 mr-1" />
                    新增脚本
                </Button>
            }
        >
            <CollectorListContent appId={Number(appId)} dictData={dictData} onEdit={(script) => {
                setEditScript(script);
                setScriptDrawerOpen(true);
            }} />

            <CollectorScriptDrawer
                appId={Number(appId)}
                script={editScript}
                open={scriptDrawerOpen}
                onOpenChange={(open) => {
                    setScriptDrawerOpen(open);
                    if (!open) setEditScript(null);
                }}
                onSuccess={onScriptSuccess}
            />
        </AppDetailNavContainer>
    );
}

interface CollectorListContentProps {
    appId: number;
    dictData: TypedDictData<["user_collector", "user_export"]>;
    onEdit: (script: CollectorScriptItemType) => void;
}

function CollectorListContent({ appId, dictData, onEdit }: CollectorListContentProps) {
    const queryClient = useQueryClient();
    const { success: showSuccess, error: showError } = useToast();
    const navigate = useNavigate();

    // 脚本状态样式映射 — 基于后端字典数据
    const scriptStatusMapper = createStatusMapper(
        {
            1: "success",
            2: "danger",
        },
        (status) => dictData.script_status?.getLabel(String(status)) || String(status),
    );

    const filterParam = Route.useSearch();
    const currentPage = filterParam.page || 1;
    const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

    // 抽屉和弹窗状态
    const [recordsDrawerOpen, setRecordsDrawerOpen] = useState(false);
    const [recordsScript, setRecordsScript] = useState<CollectorScriptItemType | null>(null);

    const [filesDrawerOpen, setFilesDrawerOpen] = useState(false);
    const [filesScript, setFilesScript] = useState<CollectorScriptItemType | null>(null);

    const [logsDrawerOpen, setLogsDrawerOpen] = useState(false);
    const [logsScript, setLogsScript] = useState<CollectorScriptItemType | null>(null);

    const [triggerDialogOpen, setTriggerDialogOpen] = useState(false);
    const [triggerScript, setTriggerScript] = useState<CollectorScriptItemType | null>(null);

    // 过滤条件
    const filters = {
        status: filterParam.status || null,
    };

    const countNumManager = usePageCountNum(filters);

    // 获取脚本列表
    const { data: scriptData, isSuccess, isLoading, isError, error } = useQuery({
        queryKey: [
            "collectorScriptList",
            appId,
            currentPage,
            currentLimit,
            filters.status,
        ],
        queryFn: ({ signal }) =>
            userCollectorScriptList(
                {
                    app_id: appId,
                    page: {
                        page: currentPage,
                        limit: currentLimit,
                    },
                    count_num: countNumManager.getCountNum(),
                    status: filters.status,
                },
                { signal }
            ),
        placeholderData: (previousData) => previousData,
    });

    isSuccess && countNumManager.handleQueryResult(scriptData);

    const scripts = getQueryResponseData<CollectorScriptItemType[]>(scriptData, []);

    // 删除脚本
    const deleteScriptMutation = useMutation({
        mutationFn: (params: { script_id: number }) =>
            userCollectorScriptDelete({ app_id: appId, script_id: params.script_id }),
        onSuccess: () => {
            showSuccess("脚本已删除");
            countNumManager.reset();
            queryClient.invalidateQueries({ queryKey: ["collectorScriptList"] });
        },
        onError: (error: any) => {
            showError(formatServerError(error));
        },
    });

    // 变更状态
    const statusMutation = useMutation({
        mutationFn: (params: { script_id: number; status: number }) =>
            userCollectorScriptStatus({ app_id: appId, script_id: params.script_id, status: params.status }),
        onSuccess: () => {
            showSuccess("状态已更新");
            queryClient.invalidateQueries({ queryKey: ["collectorScriptList"] });
        },
        onError: (error: any) => {
            showError(formatServerError(error));
        },
    });

    const refreshData = () => {
        queryClient.refetchQueries({ queryKey: ["collectorScriptList"] });
    };

    const clearCacheAndReload = () => {
        countNumManager.reset();
        queryClient.invalidateQueries({ queryKey: ["collectorScriptList"] });
    };

    const filterForm = useFilterBarForm<z.infer<typeof CollectorListFilterFormSchema>>({
        defaultValues: { status: filterParam.status },
        resolver: zodResolver(CollectorListFilterFormSchema) as any,
        initValues: { status: undefined },
        onSubmit: (data) => {
            const d = data as { status?: number };
            navigate({
                to: "/user/app/$appId/features-file/collector",
                params: { appId: appId },
                search: { status: d.status, page: 1, limit: currentLimit },
            });
        },
        onReset: () => {
            navigate({
                to: "/user/app/$appId/features-file/collector",
                params: { appId: appId },
                search: { page: 1, limit: currentLimit },
            });
        },
    });

    // 导出操作 hook（脚本记录）
    const exportRecordsAction = useUserAppExportAction({
        appId: appId,
        exportType: EXPORT_TYPE_APP_SCRIPT_RECORDS,
        params: {
            status: filters.status ?? undefined,
        },
    });

    // 表格列定义
    const columns: ColumnDef<CollectorScriptItemType>[] = [
        {
            accessorKey: "id",
            header: "ID",
            size: 60,
            cell: ({ getValue }) => (
                <div className="py-1 text-xs text-muted-foreground">{getValue<number>()}</div>
            ),
        },
        {
            accessorKey: "name",
            header: "脚本名称",
            cell: ({ getValue }) => (
                <div className="max-w-[200px] truncate py-1 text-sm font-medium" title={getValue<string>()}>
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
                        <Badge className={cn(scriptStatusMapper.getClass(status))}>
                            {scriptStatusMapper.getText(status)}
                        </Badge>
                    </div>
                );
            },
        },
        {
            accessorKey: "timeout_secs",
            header: "超时(秒)",
            size: 90,
            cell: ({ getValue }) => (
                <div className="py-1 text-sm">{getValue<number>()}</div>
            ),
        },
        {
            accessorKey: "memory_limit",
            header: "内存限制",
            size: 100,
            cell: ({ getValue }) => {
                const bytes = getValue<number>();
                if (!bytes) return <div className="py-1 text-sm">-</div>;
                const mb = (bytes / 1024 / 1024).toFixed(1);
                return <div className="py-1 text-sm">{mb} MB</div>;
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
            size: 200,
            cell: ({ row }) => {
                const script = row.original;
                const isEnabled = script.status === 1;
                return (
                    <DataTableAction className="justify-end sm:justify-center">
                        {/* 触发执行 */}
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <Button
                                variant="ghost"
                                size="sm"
                                className={cn("h-auto px-2 py-1")}
                                title="触发执行"
                                onClick={() => {
                                    setTriggerScript(script);
                                    setTriggerDialogOpen(true);
                                }}
                            >
                                <Play className="h-3 w-3" />
                                <span className="text-xs ml-1">执行</span>
                            </Button>
                        </DataTableActionItem>

                        {/* 采集记录 */}
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <Button
                                variant="ghost"
                                size="sm"
                                className={cn("h-auto px-2 py-1")}
                                title="采集记录"
                                onClick={() => {
                                    setRecordsScript(script);
                                    setRecordsDrawerOpen(true);
                                }}
                            >
                                <ScrollText className="h-3 w-3" />
                                <span className="text-xs ml-1">记录</span>
                            </Button>
                        </DataTableActionItem>

                        {/* 采集文件 */}
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <Button
                                variant="ghost"
                                size="sm"
                                className={cn("h-auto px-2 py-1")}
                                title="采集文件"
                                onClick={() => {
                                    setFilesScript(script);
                                    setFilesDrawerOpen(true);
                                }}
                            >
                                <FolderOpen className="h-3 w-3" />
                                <span className="text-xs ml-1">文件</span>
                            </Button>
                        </DataTableActionItem>

                        {/* 运行日志 */}
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <Button
                                variant="ghost"
                                size="sm"
                                className={cn("h-auto px-2 py-1")}
                                title="运行日志"
                                onClick={() => {
                                    setLogsScript(script);
                                    setLogsDrawerOpen(true);
                                }}
                            >
                                <FileText className="h-3 w-3" />
                                <span className="text-xs ml-1">日志</span>
                            </Button>
                        </DataTableActionItem>

                        {/* 编辑 */}
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <Button
                                variant="ghost"
                                size="sm"
                                className={cn("h-auto px-2 py-1")}
                                title="编辑脚本"
                                onClick={() => onEdit(script)}
                            >
                                <Eye className="h-3 w-3" />
                                <span className="text-xs ml-1">编辑</span>
                            </Button>
                        </DataTableActionItem>

                        {/* 启用/禁用 */}
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <ConfirmDialog
                                title={isEnabled ? "确认禁用" : "确认启用"}
                                description={
                                    <>
                                        您确定要{isEnabled ? "禁用" : "启用"}脚本 <strong>{script.name}</strong> 吗？
                                    </>
                                }
                                onConfirm={async () => {
                                    await statusMutation.mutateAsync({
                                        script_id: script.id,
                                        status: isEnabled ? 2 : 1,
                                    });
                                }}
                            >
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    className={cn("h-auto px-2 py-1", isEnabled ? "text-orange-500 hover:text-orange-600" : "text-green-500 hover:text-green-600")}
                                    title={isEnabled ? "禁用" : "启用"}
                                >
                                    {isEnabled ? <PowerOff className="h-3 w-3" /> : <Power className="h-3 w-3" />}
                                    <span className="text-xs ml-1">{isEnabled ? "禁用" : "启用"}</span>
                                </Button>
                            </ConfirmDialog>
                        </DataTableActionItem>

                        {/* 删除 */}
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <ConfirmDialog
                                title="确认删除"
                                description={
                                    <>
                                        您确定要删除脚本 <strong>{script.name}</strong> 吗？删除后将无法恢复。
                                    </>
                                }
                                onConfirm={async () => {
                                    await deleteScriptMutation.mutateAsync({ script_id: script.id });
                                }}
                            >
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    className={cn("h-auto px-2 py-1 text-destructive hover:text-destructive")}
                                    title="删除脚本"
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
                    <FilterBar form={filterForm} className="bg-card rounded-lg border shadow-sm relative">
                        <FilterBar.Summary>
                            <FilterTotalCount value={formatTotalCount(countNumManager.getTotal())} loading={isLoading} />
                        </FilterBar.Summary>
                        <FilterBar.MobileExtra>
                            <ExportMobileButton activeCount={exportRecordsAction.activeCount} isLoading={exportRecordsAction.activeCount > 0} onClick={exportRecordsAction.openDrawer} />
                        </FilterBar.MobileExtra>
                        {dictData.script_status && (
                            <FilterDictSelect name="status" placeholder="选择状态" label="状态" disabled={isLoading}
                                dictData={dictData.script_status} allLabel="全部"
                                className={"min-w-[100px] max-w-[130px]"} />
                        )}
                        <FilterActions>
                            <FilterSearchButton loading={isLoading} onRefreshSearch={clearCacheAndReload} />
                            <FilterResetButton loading={isLoading} />
                            <FilterBar.DesktopOnly>
                                <ExportSplitButton activeCount={exportRecordsAction.activeCount} onSubmitExport={exportRecordsAction.submit}
                                    onViewHistory={exportRecordsAction.openDrawer} isSubmitting={exportRecordsAction.isSubmitting} />
                            </FilterBar.DesktopOnly>
                        </FilterActions>
                        <FilterBar.MobileFooter>
                            {(closeDrawer) => (
                                <ExportButton isSubmitting={exportRecordsAction.isSubmitting}
                                    onSubmitExport={() => void exportRecordsAction.submit().then(closeDrawer).catch(() => {})} />
                            )}
                        </FilterBar.MobileFooter>
                    </FilterBar>
                    <ExportDrawer
                        open={exportRecordsAction.drawerOpen}
                        onOpenChange={(open) => open ? exportRecordsAction.openDrawer() : exportRecordsAction.closeDrawer()}
                        statusDict={dictData.export_task_status!}
                        tasks={exportRecordsAction.tasks} totalCount={exportRecordsAction.totalCount}
                        currentPage={exportRecordsAction.currentPage} totalPages={exportRecordsAction.totalPages}
                        onPageChange={exportRecordsAction.setPage} isLoading={exportRecordsAction.isLoadingTasks}
                    />
                </div>

                {/* 表格和分页容器 */}
                <div className="flex-1 flex flex-col min-h-0">
                    <div className="flex-1 overflow-hidden">
                        <DataTable
                            data={scripts}
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
                            <PagePagination
                                currentPage={currentPage}
                                pageSize={currentLimit}
                                total={countNumManager.getTotal() ?? 0}
                                loading={isLoading}
                                onChange={(page) => {
                                    navigate({
                                        to: "/user/app/$appId/features-file/collector",
                                        params: { appId: appId },
                                        search: {
                                            ...filterParam,
                                            page,
                                        },
                                    });
                                }}
                            />
                        )}
                    </div>
                </div>

                {/* 触发执行弹窗 */}
                {triggerScript && (
                    <CollectorTriggerDialog
                        appId={appId}
                        script={triggerScript}
                        open={triggerDialogOpen}
                        onOpenChange={setTriggerDialogOpen}
                    />
                )}

                {/* 采集记录抽屉 */}
                {recordsScript && (
                    <CollectorRecordListDrawer
                        appId={appId}
                        script={recordsScript}
                        isOpen={recordsDrawerOpen}
                        onOpenChange={setRecordsDrawerOpen}
                    />
                )}

                {/* 采集文件抽屉 */}
                {filesScript && (
                    <CollectorFilesDrawer
                        appId={appId}
                        script={filesScript}
                        isOpen={filesDrawerOpen}
                        onOpenChange={setFilesDrawerOpen}
                    />
                )}

                {/* 运行日志抽屉 */}
                {logsScript && (
                    <CollectorLogsDrawer
                        appId={appId}
                        script={logsScript}
                        isOpen={logsDrawerOpen}
                        onOpenChange={setLogsDrawerOpen}
                    />
                )}
            </div>
        </>
    );
}
