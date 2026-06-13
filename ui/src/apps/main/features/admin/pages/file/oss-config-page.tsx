import {
  adminOssConfigDelete,
  adminOssConfigList,
  type AdminOssConfigItemType,
} from "@shared/apis/admin/file";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import { cn, formatTime, TIME_STYLE } from "@shared/lib/utils";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  DataTable,
  DataTableAction,
  DataTableActionItem,
} from "@shared/components/custom/table";
import { Edit2, Plus, Trash2 } from "lucide-react";
import { useIsMobile } from "@shared/hooks/use-mobile";
import type { ColumnDef } from "@tanstack/react-table";
import { useEffect, useState } from "react";
import {
  PagePagination,
  usePageCountNum,
} from "@apps/main/lib/pagination-utils";
import { useDictData } from "@apps/main/hooks/use-dict-data";
import { OssConfigDrawer } from "./oss-config-drawer";

export function AdminOssConfigPage() {
  const queryClient = useQueryClient();
  const { success: showSuccess, error: showError } = useToast();
  const isMobile = useIsMobile();

  const [page, setPage] = useState(1);
  const pageSize = 20;
  const countNumManager = usePageCountNum({});

  const [drawerOpen, setDrawerOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<AdminOssConfigItemType | null>(
    null,
  );

  // 获取配置列表
  const {
    data: configData,
    isSuccess,
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery({
    queryKey: ["adminOssConfigList", page],
    queryFn: ({ signal }) =>
      adminOssConfigList(
        { page, limit: pageSize, count_num: countNumManager.getCountNum() },
        { signal },
      ),
  });

  useEffect(() => {
    if (isSuccess && configData) {
      countNumManager.handleQueryResult(configData);
    }
  }, [isSuccess, configData, countNumManager]);

  const configs: AdminOssConfigItemType[] = configData?.response?.data ?? [];

  // 通过 dict 获取 storage_type（含 OSS 厂商类型）
  const { dictData: adminFileDict } = useDictData(["admin_file"] as const);
  const providerTypes = (adminFileDict?.storage_type ?? []).filter(
    (item) => item.type === "oss",
  );

  const columns: ColumnDef<AdminOssConfigItemType>[] = [
    {
      accessorKey: "id",
      header: "ID",
      size: 60,
      cell: ({ getValue }) => (
        <div className={cn("font-mono text-xs")}>{getValue<number>()}</div>
      ),
    },
    {
      accessorKey: "name",
      header: "配置名称",
      cell: ({ getValue }) => (
        <div className={cn("font-medium")}>{getValue<string>()}</div>
      ),
    },
    {
      accessorKey: "config_key",
      header: "配置标识",
      cell: ({ getValue }) => (
        <div className={cn("font-mono text-xs text-muted-foreground")}>
          {getValue<string>()}
        </div>
      ),
    },
    {
      accessorKey: "provider_type",
      header: "厂商类型",
      cell: ({ getValue }) => {
        const type = getValue<string>();
        const label =
          (adminFileDict?.storage_type ?? []).find((t) => t.key === type)
            ?.val || type;
        return (
          <Badge variant="secondary" className={cn("text-xs")}>
            {label}
          </Badge>
        );
      },
    },
    {
      accessorKey: "is_private",
      header: "访问类型",
      cell: ({ getValue }) => {
        const isPrivate = getValue<boolean>();
        return (
          <Badge 
            variant={isPrivate ? "default" : "outline"} 
            className={cn("text-xs")}
          >
            {isPrivate ? "私有" : "公开"}
          </Badge>
        );
      },
    },
    {
      accessorKey: "change_time",
      header: "更新时间",
      size: 150,
      cell: ({ getValue }) => (
        <div className={cn("text-xs text-muted-foreground")}>
          {getValue<number>()
            ? formatTime(getValue<number>(), TIME_STYLE.RELATIVE_ELEMENT)
            : "-"}
        </div>
      ),
    },
    {
      id: "actions",
      header: "操作",
      size: 100,
      cell: ({ row }) => {
        const item = row.original;
        return (
          <DataTableAction
            className={cn(isMobile ? "justify-end" : "justify-center")}
          >
            <DataTableActionItem
              mobileDisplay="display"
              desktopDisplay="collapsed"
            >
              <Button
                variant="ghost"
                size="sm"
                className={cn("h-7 px-2")}
                onClick={() => handleOpenEdit(item)}
              >
                <Edit2 className={cn("h-4 w-4")} />
                <span className={cn("ml-2")}>编辑</span>
              </Button>
            </DataTableActionItem>
            <DataTableActionItem
              mobileDisplay="display"
              desktopDisplay="collapsed"
            >
              <ConfirmDialog
                title="确认删除"
                description={
                  <>
                    您确定要删除 OSS 配置 <strong>{item.name}</strong>（
                    {item.config_key}）吗？如有文件正在使用此配置将无法删除。
                  </>
                }
                onConfirm={async () => {
                  await deleteMutation.mutateAsync(item.id);
                }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn(
                    "h-7 px-2 text-destructive hover:text-destructive",
                  )}
                >
                  <Trash2 className={cn("h-4 w-4")} />
                  <span className={cn("ml-2")}>删除</span>
                </Button>
              </ConfirmDialog>
            </DataTableActionItem>
          </DataTableAction>
        );
      },
    },
  ];

  // 删除配置
  const deleteMutation = useMutation({
    mutationFn: (id: number) => adminOssConfigDelete({ id }),
    onSuccess: () => {
      showSuccess("配置已删除");
      queryClient.invalidateQueries({ queryKey: ["adminOssConfigList"] });
    },
    onError: (err: any) => {
      showError(err?.data?.message || err?.message || "删除失败");
    },
  });

  const handleOpenAdd = () => {
    setEditingItem(null);
    setDrawerOpen(true);
  };

  const handleOpenEdit = (item: AdminOssConfigItemType) => {
    setEditingItem(item);
    setDrawerOpen(true);
  };

  const handleSaveSuccess = () => {
    setDrawerOpen(false);
    setEditingItem(null);
    queryClient.invalidateQueries({ queryKey: ["adminOssConfigList"] });
  };

  return (
    <div
      className={cn(
        "container mx-auto p-4 lg:px-6 py-5 max-w-[1600px] flex flex-col min-h-0 space-y-5",
      )}
    >
      {/* 顶部操作 */}
      <div className={cn("flex items-center justify-between")}>
        <div className={cn("text-sm text-muted-foreground")}>
          管理 OSS 云存储配置，配置后可在文件上传时选择对应存储
        </div>
        <Button size="sm" onClick={handleOpenAdd}>
          <Plus className={cn("h-4 w-4 mr-1")} />
          新增配置
        </Button>
      </div>

      {/* 配置列表 */}
      <div className={cn("flex-1 min-h-0")}>
        <DataTable
          data={configs}
          columns={columns}
          loading={isLoading}
          error={
            isError ? (
              <CenteredError
                error={error}
                variant="content"
                onReset={() => refetch()}
              />
            ) : null
          }
          emptyComponent={
            <div className={cn("text-center text-muted-foreground py-16")}>
              <p>暂无 OSS 配置</p>
              <p className={cn("text-xs mt-1")}>点击右上角「新增配置」添加</p>
            </div>
          }
          scrollSnapDelay={300}
          className={cn(
            "[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full",
          )}
          tableContainerClassName={cn("h-full")}
        />
      </div>

      {/* 分页 */}
      {countNumManager.hasTotal() && (
        <div className={cn("flex justify-end pt-2")}>
          <PagePagination
            currentPage={page}
            pageSize={pageSize}
            total={countNumManager.getTotal() ?? 0}
            loading={isLoading}
            onChange={(newPage) => setPage(newPage)}
            showTotal={true}
            showPageSize={false}
            showSizeCount={5}
          />
        </div>
      )}

      {/* 新增/编辑抽屉 */}
      <OssConfigDrawer
        open={drawerOpen}
        onOpenChange={setDrawerOpen}
        config={editingItem}
        providerTypes={providerTypes}
        onSuccess={handleSaveSuccess}
      />
    </div>
  );
}
