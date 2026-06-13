import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav";
import { useDictData } from "@apps/main/hooks/use-dict-data";
import { Route } from "@apps/main/routes/_main/user/app/$appId/features-file/list";
import { type UserFileItemType } from "@shared/apis/user/file";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { Button } from "@shared/components/ui/button";
import { Download, Link, Upload } from "lucide-react";
import React, { useState } from "react";
import { featureFileModuleConfig } from "../nav-info";
import { FileUploadDialog } from "./file-upload-dialog";
import { FileUrlDownloadDialog } from "./file-url-download-dialog";
import { FileListDownloadingView } from "./list-downloading-view";
import { FileListLineageView } from "./list-lineage-view";
import { FileListNormalView } from "./list-normal-view";
import { useNavigate, useRouter } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";

export default function AppDetailFeatureFileListPage() {
  const { appId } = Route.useParams();
  const navigate = useNavigate();
  const router = useRouter();
  const queryClient = useQueryClient();

  const search = Route.useSearch();
  const mode = search.mode ?? "normal";

  // sourceFile 保存在 state 中：点击行时设置。页面刷新后为 null（仅 source_id 保留在 URL）
  const [sourceFile, setSourceFile] = useState<UserFileItemType | null>(null);

  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["user_file", "user_export"] as const);

  if (dictError && dictErrors.length > 0) {
    return <CenteredError variant="page" error={dictErrors} onReset={refetchDict} />;
  }

  if (dictIsLoading) {
    return <PageSkeletonTable variant="page" />;
  }

  // 导航到普通视图（清除所有模式参数）
  const goToNormal = () =>
    navigate({
      to: "/user/app/$appId/features-file/list",
      params: { appId },
      search: {},
    });

  // 导航到下载中视图
  const goToDownloading = () =>
    navigate({
      to: "/user/app/$appId/features-file/list",
      params: { appId },
      search: { mode: "downloading" },
    });

  // 导航到关联文件视图（点击行的关联图标时调用）
  const goToLineage = (file: UserFileItemType, relType?: number | null) => {
    setSourceFile(file);
    navigate({
      to: "/user/app/$appId/features-file/list",
      params: { appId },
      search: {
        mode: "lineage",
        source_id: file.id,
        rel_type: relType ?? undefined,
      },
    });
  };

  // 在关联视图内切换关联类型（replace: 不推历史记录）
  const onRelTypeChange = (relType?: number | null) =>
    navigate({
      to: "/user/app/$appId/features-file/list",
      params: { appId },
      search: {
        mode: "lineage" as const,
        source_id: search.source_id,
        rel_type: relType ?? undefined,
        pos: undefined,
      },
      replace: true,
    });

  const refreshFileQueries = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ["userFileList"], refetchType: "all" }),
      queryClient.refetchQueries({ queryKey: ["userFileList"], type: "all" }),
      queryClient.invalidateQueries({ queryKey: ["userFileDownloadingList"], refetchType: "all" }),
      queryClient.refetchQueries({ queryKey: ["userFileDownloadingList"], type: "all" }),
    ]);
  };

  // 上传/下载成功后刷新普通视图
  const onUploadSuccess = () => {
    navigate({
      to: "/user/app/$appId/features-file/list",
      params: { appId },
      search: {},
      replace: true,
    });
    void refreshFileQueries();
  };

  const onDownloadSuccess = () => {
    navigate({
      to: "/user/app/$appId/features-file/list",
      params: { appId },
      search: {},
      replace: true,
    });
    void refreshFileQueries();
  };

  return (
    <AppDetailNavContainer
      {...featureFileModuleConfig}
      actions={
        mode === "normal" ? (
          <div className="flex flex-wrap items-center gap-2">
            <FileUploadDialog appId={Number(appId)} uploadConfig={dictData} onSuccess={onUploadSuccess}>
              <Button size="sm" variant="default">
                <Upload className="h-4 w-4 mr-1" />
                上传文件
              </Button>
            </FileUploadDialog>
            <FileUrlDownloadDialog appId={Number(appId)} onSuccess={onDownloadSuccess}>
              <Button size="sm" variant="outline">
                <Link className="h-4 w-4 mr-1" />
                URL 下载
              </Button>
            </FileUrlDownloadDialog>
            <Button size="sm" variant="outline" onClick={goToDownloading} className="group">
              <Download className="mr-1 h-4 w-4 transition-transform duration-200 motion-reduce:transform-none group-hover:animate-bounce" />
              下载中
            </Button>
          </div>
        ) : null
      }
    >
      {mode === "normal" && (
        <FileListNormalView
          appId={Number(appId)}
          dictData={dictData}
          onSwitchToLineageView={goToLineage}
        />
      )}

      {mode === "downloading" && (
        <FileListDownloadingView
          appId={Number(appId)}
          onGoToRoot={goToNormal}
        />
      )}

      {mode === "lineage" && search.source_id != null && (
        <FileListLineageView
          appId={Number(appId)}
          dictData={dictData}
          sourceFileId={search.source_id}
          sourceFile={sourceFile}
          relTypeFilter={search.rel_type}
          onSwitchToLineageView={goToLineage}
          onRelTypeChange={onRelTypeChange}
          onGoBack={() => router.history.back()}
          onGoToRoot={goToNormal}
        />
      )}
    </AppDetailNavContainer>
  );
}

