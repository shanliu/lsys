"use client";

import {
    systemSenderMailerConfigDel,
    systemSenderMailerConfigList,
    systemSenderMailerMapping,
} from "@shared/apis/admin/sender-mailer";
import { SenderRuleConfigView } from "@apps/main/components/local/sender-config/rule-config-view";
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import {
    DEFAULT_PAGE_SIZE,
    PagePagination,
} from "@apps/main/lib/pagination-utils";
import { Route } from "@apps/main/routes/_main/admin/email/send-config/rule";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import { EmailSendConfigNavContainer } from "@apps/main/features/admin/components/ui/email-send-config-nav";
import { useIsMobile } from "@shared/hooks/use-mobile";
import { cn, formatServerError, formatTime, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils";
import { DictItemType } from "@shared/types/apis-dict";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ColumnDef } from "@tanstack/react-table";
import { Plus, Trash2 } from "lucide-react";
import React from "react";
import { emailSendConfigModuleConfig } from "../nav-info";
import { EmailSendConfigRuleDrawer } from "./send-config-rule-drawer";

interface EmailSendConfigRuleConfigItem {
    id: number;
    add_time: number;
    config_data: string;
    config_type: string;
    priority: number;
}

export function EmailSendConfigRulePage() {
    const [configDrawerOpen, setConfigDrawerOpen] = React.useState(false);

    // 字典数据获取
    const {
        data: mappingData,
        isLoading: dictIsLoading,
        isError: dictError,
        error: dictErrors,
        refetch: refetchDict,
    } = useQuery({
        queryKey: ["admin-sender-mailer-mapping"],
        queryFn: async ({ signal }) => {
            const result = await systemSenderMailerMapping({ signal });
            return result;
        },
    });

    const dictData = mappingData?.response || null;

    // 如果字典加载失败，显示错误页面
    if (dictError && dictErrors) {
        return (
            <EmailSendConfigNavContainer  className={cn("m-4 md:m-6")} {...emailSendConfigModuleConfig}>
                <CenteredError
                    variant="content"
                    error={dictErrors}
                    onReset={refetchDict}
                />
            </EmailSendConfigNavContainer>
        );
    }

    // 如果字典加载中，显示骨架屏
    if (dictIsLoading || !dictData) {
        return (
            <EmailSendConfigNavContainer  className={cn("m-4 md:m-6")} {...emailSendConfigModuleConfig}>
                <PageSkeletonTable variant="content" />
            </EmailSendConfigNavContainer>
        );
    }

    // 字典加载成功，渲染内容组件
    return (
        <>
            <EmailSendConfigNavContainer
             className={cn("m-4 md:m-6")}  
                {...emailSendConfigModuleConfig}
                actions={
                    <Button
                        size="sm"
                        variant="outline"
                        onClick={() => setConfigDrawerOpen(true)}
                    >
                        <Plus className={cn("mr-2 h-4 w-4")} />
                        新增配置
                    </Button>
                }
            >
                <EmailSendConfigSendRuleContent dictData={dictData} />
            </EmailSendConfigNavContainer>

            <EmailSendConfigRuleDrawer
                open={configDrawerOpen}
                onOpenChange={setConfigDrawerOpen}
            />
        </>
    );
}

interface EmailSendConfigSendRuleContentProps {
    dictData: any;
}

export function EmailSendConfigSendRuleContent({ dictData }: EmailSendConfigSendRuleContentProps) {
    const toast = useToast();
    const queryClient = useQueryClient();
    const isMobile = useIsMobile();

    const currentPage = 1;
    const currentLimit = DEFAULT_PAGE_SIZE;

    // 获取邮件配置列表数据
    const {
        data: configData,
        isLoading,
        isError,
        error,
    } = useQuery({
        queryKey: ["admin-mail-config-list"],
        queryFn: async ({ signal }) => {
            const result = await systemSenderMailerConfigList({ signal });
            return result;
        },
        placeholderData: (previousData) => previousData,
    });

    // 从查询结果中提取数据
    const configs = getQueryResponseData<EmailSendConfigRuleConfigItem[]>(configData, []);

    // 删除配置
    const deleteMutation = useMutation({
        mutationFn: (id: number) => systemSenderMailerConfigDel({ config_id: id }),
        onSuccess: () => {
            toast.success("配置删除成功");
            queryClient.invalidateQueries({ queryKey: ["admin-mail-config-list"] });
        },
        onError: (error: any) => {
            toast.error(formatServerError(error));
        },
    });

    const handleDelete = async (id: number) => {
        await deleteMutation.mutateAsync(id);
    };

    // 刷新数据
    const refreshData = () => {
        queryClient.refetchQueries({ queryKey: ["admin-mail-config-list"] });
    };

    // 定义表格列配置
    const columns: ColumnDef<EmailSendConfigRuleConfigItem>[] = [
        {
            accessorKey: "id",
            header: () => <div className={cn(isMobile ? "" : "text-right")}>ID</div>,
            size: 80,
            cell: ({ getValue }) => (
                <div className={cn("font-mono text-sm", isMobile ? "" : "text-right")}>{getValue<number>()}</div>
            ),
        },
        {
            accessorKey: "config_type",
            header: "配置类型",
            cell: ({ getValue }) => {
                const configType = getValue<string>();
                const typeLabel = dictData.mail_config_type.find((item: DictItemType) => item.key === configType)?.val || configType;
                return <Badge>{typeLabel}</Badge>;
            },
        },
        {
            accessorKey: "priority",
            header: () => <div className={cn(isMobile ? "" : "text-right")}>优先级</div>,
            size: 100,
            cell: ({ getValue }) => (
                <div className={cn("font-mono text-sm", isMobile ? "" : "text-right")}>{getValue<number>()}</div>
            ),
        },
        {
            accessorKey: "config_data",
            header: "配置数据",
            cell: ({ row }) => {
                const config = row.original;
                return (
                    <SenderRuleConfigView
                        configType={Number(config.config_type)}
                        configData={config.config_data}
                        displayType="table"
                    />
                );
            },
        },
        {
            accessorKey: "add_time",
            header: "添加时间",
            size: 180,
            cell: ({ getValue }) => (
                <div className="text-sm text-muted-foreground">
                    {formatTime(getValue<number>(), TIME_STYLE.ABSOLUTE_TEXT)}
                </div>
            ),
        },
        {
            id: "actions",
            header: () => <div className="text-center">操作</div>,
            size: 80,
            cell: ({ row }) => {
                const config = row.original;

                return (
                    <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="display">
                            <ConfirmDialog
                                title="确认删除"
                                description={
                                    <div>
                                        确定要删除配置 <strong>ID: {config.id}</strong> 吗？
                                        <br />
                                        删除后无法恢复。
                                    </div>
                                }
                                onConfirm={() => handleDelete(Number(config.id))}
                            >
                                <Button
                                    variant="ghost"
                                    className={cn("h-7 px-2 ")}
                                    title="删除"
                                >
                                    <Trash2 className="h-4 w-4" />
                                    {isMobile ? <span className="ml-2">删除</span> : ""}
                                </Button>
                            </ConfirmDialog>
                        </DataTableActionItem>
                    </DataTableAction>
                );
            },
        },
    ];

    return (
        <div className="flex flex-col min-h-0 space-y-3">
            {/* 表格和分页容器 - 确保不超出页面高度 */}
            <div className="flex-1 flex flex-col min-h-0">
                {/* 数据表格 - 使用 flex-1 但不设置 min-h-0，让分页有足够空间 */}
                <div className="flex-1 overflow-hidden">
                    <DataTable
                        data={configs}
                        columns={columns}
                        loading={isLoading}
                        error={
                            isError ? (
                                <CenteredError error={error} variant="content" onReset={refreshData} />
                            ) : null
                        }
                        scrollSnapDelay={300}
                        className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
                        tableContainerClassName="h-full"
                    />
                </div>

            </div>
        </div>
    );
}
