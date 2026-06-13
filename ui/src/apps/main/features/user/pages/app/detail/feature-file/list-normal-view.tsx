import { FilterBar } from "@apps/main/components/filter-bar/container";
import { useFilterBarForm } from "@apps/main/hooks/use-filter-bar-form";
import { FilterActions } from "@apps/main/components/filter-bar/filter-actions/filter-actions";
import { FilterSearchButton } from "@apps/main/components/filter-bar/filter-actions/filter-search-button";
import { FilterResetButton } from "@apps/main/components/filter-bar/filter-actions/filter-reset-button";
import {
  FilterContentSearch,
  FilterDictSelect,
  FilterTagCombobox,
  FilterTotalCount,
} from "@apps/main/components/filter-bar/filter-fields";
import {
  ExportButton,
  ExportMobileButton,
  ExportSplitButton,
} from "@apps/main/components/export-manager/export-buttons";
import { ExportDrawer } from "@apps/main/components/export-manager/export-drawer";
import { useUserAppExportAction } from "@apps/main/hooks/use-user-app-export-action";
import { type TypedDictData } from "@apps/main/hooks/use-dict-data";
import {
  CursorPagination,
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  useLimitCountNum,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-file/list";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  userFileDelete,
  userFileList,
  userFileTagNames,
  EXPORT_TYPE_APP_FILE_LIST,
  type UserFileItemType,
} from "@shared/apis/user/file";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { useToast } from "@shared/contexts/toast-context";
import { formatTotalCount } from "@shared/lib/utils/format-utils";
import { getQueryResponseCursor, getQueryResponseData } from "@shared/lib/utils";
import { type LimitType } from "@shared/types/base-schema";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";
import { z } from "zod";
import { FileListTable } from "./file-list-table";
import { CONTENT_SEARCH_TYPES, FileListFilterFormSchema } from "./list-schema";

interface FileListNormalViewProps {
  appId: number;
  dictData: TypedDictData<["user_file", "user_export"]>;
  onSwitchToLineageView: (file: UserFileItemType, relType?: number | null) => void;
}

export function FileListNormalView({
  appId,
  dictData,
  onSwitchToLineageView,
}: FileListNormalViewProps) {
  const queryClient = useQueryClient();
  const { error: showError } = useToast();
  const navigate = useNavigate();

  const filterParam = Route.useSearch();
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  const filters = {
    status: filterParam.status || null,
    tag_name: filterParam.tag_name || null,
    content_type: filterParam.content_type || null,
    content_value: filterParam.content_value || null,
  };

  const pagination: LimitType = {
    pos: filterParam.pos || null,
    limit: currentLimit,
    forward: filterParam.forward ?? true,
    more: true,
  };

  const searchGo = useSearchNavigate(navigate, filterParam);
  const countNumManager = useLimitCountNum(filters);

  const filterForm = useFilterBarForm<z.infer<typeof FileListFilterFormSchema>>({
    defaultValues: {
      status: filterParam.status?.toString() as any,
      tag_name: filterParam.tag_name,
      content_type: filterParam.content_type,
      content_value: filterParam.content_value,
    },
    resolver: zodResolver(FileListFilterFormSchema) as any,
    initValues: { status: undefined, tag_name: undefined, content_type: undefined, content_value: undefined },
    onSubmit: (data) => {
      searchGo({
        status: data.status,
        tag_name: data.tag_name,
        content_type: data.content_type,
        content_value: data.content_type ? data.content_value : undefined,
        pos: null,
        forward: true,
      });
    },
    onReset: () => {
      searchGo({ pos: null, limit: currentLimit, forward: true, status: undefined, tag_name: undefined, content_type: undefined, content_value: undefined });
    },
  });

  const contentFilterParams = useMemo(() => {
    if (!filters.content_type || !filters.content_value) return {};
    const map: Record<string, string> = { file_md5: "file_md5", source_url: "source_url", url: "url" };
    const key = map[filters.content_type];
    return key ? { [key]: filters.content_value } : {};
  }, [filters.content_type, filters.content_value]);

  const exportAction = useUserAppExportAction({
    appId,
    exportType: EXPORT_TYPE_APP_FILE_LIST,
    params: {
      status: filters.status ?? undefined,
      tag_names: filters.tag_name ? [filters.tag_name] : undefined,
      ...contentFilterParams,
    },
    onViewFile: (taskId) =>
      navigate({
        to: "/user/app/$appId/features-file/list",
        params: { appId: appId as any },
        search: { tag_name: `export_${taskId}` } as any,
      }),
  });

  const { data: fileData, isLoading, isError, error, isSuccess } = useQuery({
    queryKey: [
      "userFileList", appId,
      pagination.pos, currentLimit, pagination.forward, pagination.more,
      filters.status, filters.tag_name, filters.content_type, filters.content_value,
    ],
    queryFn: ({ signal }) => {
      const canHaveAttrs = !filters.status || filters.status === 1;
      return userFileList({
        app_id: appId,
        limit: pagination,
        count_num: countNumManager.getCountNum(),
        status: filters.status ?? undefined,
        tag_names: filters.tag_name ? [filters.tag_name] : undefined,
        attr_tag: canHaveAttrs ? true : undefined,
        attr_lineage: canHaveAttrs ? true : undefined,
        ...contentFilterParams,
      }, { signal });
    },
    placeholderData: (prev) => prev,
  });

  isSuccess && countNumManager.handleQueryResult(fileData as any);

  const files = useMemo(() => getQueryResponseData<UserFileItemType[]>(fileData, []), [fileData]);
  const cursorData = getQueryResponseCursor(fileData);

  const deleteFileMutation = useMutation({
    mutationFn: (id: number) => userFileDelete({ file_ref_id: id }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["userFileList"] }),
    onError: (err: any) => { showError(err?.data?.message || err?.message || "删除文件失败"); },
  });

  const fetchTagNamesFn = useCallback(async (prefix: string, signal: AbortSignal) => {
    const res = await queryClient.fetchQuery({
      queryKey: ["userFileTagNames", appId, prefix],
      queryFn: () => userFileTagNames({ app_id: appId, tag_name_prefix: prefix || undefined, limit: 5 }, { signal }),
      staleTime: 30_000,
    });
    return res?.response?.data ?? [];
  }, [queryClient, appId]);

  const refreshData = () => queryClient.refetchQueries({ queryKey: ["userFileList"] });
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["userFileList"] });
  };
  const goToDownloadingView = () => {
    navigate({
      to: "/user/app/$appId/features-file/list",
      params: { appId },
      search: { mode: "downloading" },
    });
  };

  return (
    <div className="flex flex-col min-h-0 space-y-3">
      <div className="flex-shrink-0 mb-1 sm:mb-4">
        <FilterBar form={filterForm} className="bg-card rounded-lg border shadow-sm relative">
          <FilterBar.Summary>
            <FilterTotalCount value={formatTotalCount(countNumManager.getTotalInfo())} loading={isLoading} />
          </FilterBar.Summary>
          <FilterBar.MobileExtra>
            <ExportMobileButton activeCount={exportAction.activeCount} isLoading={exportAction.activeCount > 0} onClick={exportAction.openDrawer} />
          </FilterBar.MobileExtra>
          <FilterDictSelect name="status" placeholder="选择状态" label="状态" disabled={isLoading}
            dictData={dictData.file_status} allLabel="全部" className="min-w-[100px] max-w-[130px]" />
          <FilterTagCombobox name="tag_name" placeholder="选择标签" searchPlaceholder="搜索标签..." label="标签"
            disabled={isLoading} allLabel="全部"
            fetchTagNames={fetchTagNamesFn}
          />
          <FilterContentSearch typeName="content_type" valueName="content_value" options={CONTENT_SEARCH_TYPES}
            label="文件内容" typePlaceholder="选择类型"
            valuePlaceholder={(type) => ({ file_md5: "输入文件MD5", source_url: "输入来源URL", url: "输入本地URL" })[type] || "请输入..."}
            disabled={isLoading} />
          <FilterActions>
            <FilterSearchButton loading={isLoading} onRefreshSearch={clearCacheAndReload} />
            <FilterResetButton loading={isLoading} />
            <FilterBar.DesktopOnly>
              <ExportSplitButton activeCount={exportAction.activeCount} onSubmitExport={exportAction.submit}
                onViewHistory={exportAction.openDrawer} isSubmitting={exportAction.isSubmitting} />
            </FilterBar.DesktopOnly>
          </FilterActions>
          <FilterBar.MobileFooter>
            {(closeDrawer) => (
              <ExportButton isSubmitting={exportAction.isSubmitting}
                onSubmitExport={() => void exportAction.submit().then(closeDrawer).catch(() => {})} />
            )}
          </FilterBar.MobileFooter>
        </FilterBar>
        <ExportDrawer
          open={exportAction.drawerOpen}
          onOpenChange={(open) => open ? exportAction.openDrawer() : exportAction.closeDrawer()}
          statusDict={dictData.export_task_status!}
          tasks={exportAction.tasks} totalCount={exportAction.totalCount}
          currentPage={exportAction.currentPage} totalPages={exportAction.totalPages}
          onPageChange={exportAction.setPage} isLoading={exportAction.isLoadingTasks}
          onViewFile={exportAction.handleViewFile}
        />
      </div>

      <div className="flex-1 flex flex-col min-h-0">
        <div className="flex-1 overflow-hidden">
          <FileListTable
            appId={appId} dictData={dictData} data={files} loading={isLoading}
            error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : undefined}
            onGoToDownloadingPage={goToDownloadingView}
            onSwitchToLineageView={onSwitchToLineageView}
            onDeleteFile={(id) => deleteFileMutation.mutateAsync(id)}
            onTagsChanged={() => queryClient.invalidateQueries({ queryKey: ["userFileList"] })}
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
