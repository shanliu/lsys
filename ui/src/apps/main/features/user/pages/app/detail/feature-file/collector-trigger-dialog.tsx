import {
    userCollectorSubmitTask,
    type CollectorScriptItemType,
} from "@shared/apis/user/collector";
import { Button } from "@shared/components/ui/button";
import { ContentDialog } from "@shared/components/custom/dialog/content-dialog";
import { useToast } from "@shared/contexts/toast-context";
import { cn, formatServerError } from "@shared/lib/utils";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Loader2, Play } from "lucide-react";
import { useEffect, useState } from "react";
import { AsyncCodeEditor } from "./async-code-editor";

interface CollectorTriggerDialogProps {
    appId: number;
    script: CollectorScriptItemType;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function CollectorTriggerDialog({
    appId,
    script,
    open,
    onOpenChange,
}: CollectorTriggerDialogProps) {
    const toast = useToast();
    const [paramsCode, setParamsCode] = useState("{}");
    const queryClient = useQueryClient();

    // 重置参数当弹窗打开时
    useEffect(() => {
        if (open) {
            setParamsCode("{}");
        }
    }, [open]);

    const submitMutation = useMutation({
        mutationFn: () => {
            let params: any = undefined;
            try {
                const trimmed = paramsCode.trim();
                if (trimmed && trimmed !== "{}") {
                    params = JSON.parse(trimmed);
                }
            } catch {
                throw new Error("参数JSON格式错误，请检查输入");
            }

            return userCollectorSubmitTask({
                app_id: appId,
                script_id: script.id,
                params,
            });
        },
        onSuccess: (result) => {
            const data = result?.response;
            toast.success(
                `任务已提交！请求ID: ${data?.request_id || "-"}, 记录ID: ${data?.record_id || "-"}`
            );
            // 清理与采集记录相关的列表缓存：记录、日志、文件
            queryClient.invalidateQueries({ queryKey: ["collectorRecordList"] });
            queryClient.invalidateQueries({ queryKey: ["collectorRecordLogList"] });
            queryClient.invalidateQueries({ queryKey: ["collectorRecordFileList"] });

            onOpenChange(false);
        },
        onError: (error: any) => {
            toast.error(formatServerError(error));
        },
    });

    return (
        <ContentDialog
            open={open}
            onOpenChange={onOpenChange}
            title="触发执行"
            className="sm:max-w-[600px]"
            content={
                <div>
                    <div className="text-sm font-medium">脚本: <strong>{script.name}</strong> (ID: {script.id})</div>
                    <div className="space-y-3 py-4">
                        <div className="text-sm font-medium">执行参数 (JSON)</div>
                        <AsyncCodeEditor
                            value={paramsCode}
                            language="json"
                            placeholder='请输入执行参数，JSON格式，如 {"key": "value"}'
                            onChange={setParamsCode}
                            minHeight={200}
                        />
                        <p className="text-xs text-muted-foreground">
                            输入传递给脚本的参数，必须为有效的 JSON 格式。留空或 {"{}"} 表示无参数。
                        </p>
                    </div>
                </div>
            }
            footer={(close) => (
                <>
                    <Button
                        variant="outline"
                        onClick={() => {
                            onOpenChange(false);
                        }}
                        disabled={submitMutation.isPending}
                    >
                        取消
                    </Button>
                    <Button
                        onClick={() => submitMutation.mutate()}
                        disabled={submitMutation.isPending}
                        className="w-full sm:w-auto"
                    >
                        {submitMutation.isPending ? (
                            <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />
                        ) : (
                            <Play className="mr-2 h-4 w-4" />
                        )}
                        执行
                    </Button>
                </>
            )}
        >
            <></>
        </ContentDialog>
    );
}
