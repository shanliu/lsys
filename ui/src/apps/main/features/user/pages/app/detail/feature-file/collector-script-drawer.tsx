import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { zodResolver } from "@hookform/resolvers/zod";
import {
    userCollectorScriptAdd,
    userCollectorScriptDetail,
    userCollectorScriptEdit,
    type CollectorScriptItemType,
} from "@shared/apis/user/collector";
import { Button } from "@shared/components/ui/button";
import {
    Form,
    FormControl,
    FormDescription,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
} from "@shared/components/ui/form";
import { Input } from "@shared/components/ui/input";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "@shared/components/ui/popover";
import { useToast } from "@shared/contexts/toast-context";
import { cn, formatServerError } from "@shared/lib/utils";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Info, Loader2 } from "lucide-react";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { AsyncCodeEditor } from "./async-code-editor";

// 脚本表单校验
const CollectorScriptFormSchema = z.object({
    name: z.string().min(1, "脚本名称不能为空"),
    script_code: z.string().min(1, "脚本代码不能为空"),
    timeout_secs: z.coerce.number().min(1, "超时时间至少1秒").optional(),
    memory_limit: z.coerce.number().min(0, "内存限制不能为负数").optional(),
});
type CollectorScriptFormType = z.infer<typeof CollectorScriptFormSchema>;

interface CollectorScriptDrawerProps {
    appId: number;
    script?: CollectorScriptItemType | null;
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSuccess?: () => void;
}

export function CollectorScriptDrawer({
    appId,
    script,
    open,
    onOpenChange,
    onSuccess,
}: CollectorScriptDrawerProps) {
    const toast = useToast();
    const queryClient = useQueryClient();
    const isEdit = !!script;

    const form = useForm<CollectorScriptFormType>({
        resolver: zodResolver(CollectorScriptFormSchema),
        defaultValues: {
            name: "",
            script_code: "",
            timeout_secs: 30,
            memory_limit: 0,
        },
    });

    // 编辑时从后端获取脚本详情（含 script_code）
    const detailQuery = useQuery({
        queryKey: ["collectorScriptDetail", appId, script?.id],
        queryFn: ({ signal }) =>
            userCollectorScriptDetail(
                { app_id: appId, script_id: script!.id },
                { signal }
            ),
        enabled: open && isEdit,
    });

    // 填充表单
    useEffect(() => {
        if (open && script && detailQuery.data) {
            const detail = detailQuery.data.response;
            form.reset({
                name: detail?.name ?? script.name,
                script_code: detail?.script_code ?? "",
                timeout_secs: detail?.timeout_secs ?? script.timeout_secs,
                memory_limit: detail?.memory_limit ?? script.memory_limit,
            });
        } else if (open && !script) {
            form.reset({
                name: "",
                script_code: "",
                timeout_secs: 30,
                memory_limit: 0,
            });
        }
    }, [open, script, detailQuery.data, form]);

    const mutation = useMutation({
        mutationFn: (data: CollectorScriptFormType) =>
            isEdit
                ? userCollectorScriptEdit({
                    app_id: appId,
                    script_id: script!.id,
                    name: data.name,
                    script_code: data.script_code,
                    timeout_secs: data.timeout_secs,
                    memory_limit: data.memory_limit,
                })
                : userCollectorScriptAdd({
                    app_id: appId,
                    name: data.name,
                    script_code: data.script_code,
                    timeout_secs: data.timeout_secs,
                    memory_limit: data.memory_limit,
                }),
        onSuccess: () => {
            toast.success(isEdit ? "脚本更新成功" : "脚本添加成功");
            queryClient.invalidateQueries({ queryKey: ["collectorScriptList"] });
            onSuccess?.();
        },
        onError: (error: any) => {
            toast.error(formatServerError(error));
        },
    });

    const onSubmit = (data: CollectorScriptFormType) => {
        mutation.mutate(data);
    };

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent className="w-[95%] md:w-[700px] lg:w-[800px]">
                <DrawerHeader>
                    <DrawerTitle>{isEdit ? "编辑脚本" : "新增脚本"}</DrawerTitle>
                    <DrawerDescription>
                        {isEdit ? "修改采集脚本信息及代码" : "填写采集脚本信息及代码"}
                    </DrawerDescription>
                </DrawerHeader>

                {isEdit && detailQuery.isLoading && (
                    <div className="flex items-center justify-center py-8">
                        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                        <span className="ml-2 text-sm text-muted-foreground">加载脚本数据...</span>
                    </div>
                )}

                {isEdit && detailQuery.isError && (
                    <div className="text-center py-8 text-sm text-destructive">
                        加载脚本详情失败: {formatServerError(detailQuery.error)}
                    </div>
                )}

                <Form {...form}>
                    <form onSubmit={form.handleSubmit(onSubmit)} className={cn("space-y-4 mt-6", isEdit && detailQuery.isLoading && "opacity-50 pointer-events-none")}>
                        <FormField
                            control={form.control}
                            name="name"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>脚本名称</FormLabel>
                                    <FormControl>
                                        <Input placeholder="请输入脚本名称" {...field} />
                                    </FormControl>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <div className="grid grid-cols-2 gap-4">
                            <FormField
                                control={form.control}
                                name="timeout_secs"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>超时时间(秒)</FormLabel>
                                        <FormControl>
                                            <Input
                                                type="number"
                                                placeholder="30"
                                                {...field}
                                                onChange={(e) => field.onChange(Number(e.target.value))}
                                            />
                                        </FormControl>
                                        <FormDescription>脚本执行的最大超时秒数</FormDescription>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />

                            <FormField
                                control={form.control}
                                name="memory_limit"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>内存限制(字节)</FormLabel>
                                        <FormControl>
                                            <Input
                                                type="number"
                                                placeholder="0"
                                                {...field}
                                                onChange={(e) => field.onChange(Number(e.target.value))}
                                            />
                                        </FormControl>
                                        <FormDescription>0 表示不限制，单位为字节</FormDescription>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />
                        </div>

                        <FormField
                            control={form.control}
                            name="script_code"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>
                                        <span className="flex items-center gap-1">
                                            脚本代码
                                            <Popover>
                                                <PopoverTrigger asChild>
                                                    <Info className="h-3.5 w-3.5 cursor-pointer text-muted-foreground hover:text-foreground" />
                                                </PopoverTrigger>
                                                <PopoverContent className="w-[460px] text-sm" side="right" align="start">
                                                    <p className="font-semibold mb-2">调用 <code>runtime.std.initGlobalsEnv()</code> 后可使用以下全局变量：</p>
                                                    <ul className="space-y-1 mb-3">
                                                        <li><code className="text-xs bg-muted px-1 rounded">params</code> — 任务入参对象</li>
                                                        <li><code className="text-xs bg-muted px-1 rounded">getEnv</code> — 获取环境变量函数</li>
                                                        <li><code className="text-xs bg-muted px-1 rounded">fetch</code> — Fetch API</li>
                                                        <li><code className="text-xs bg-muted px-1 rounded">fs</code> — Node.js 风格文件操作（readFileSync / writeFileSync 等）</li>
                                                        <li><code className="text-xs bg-muted px-1 rounded">console</code> — 控制台输出</li>
                                                        <li><code className="text-xs bg-muted px-1 rounded">crypto</code> — Web Crypto API</li>
                                                        <li><code className="text-xs bg-muted px-1 rounded">btoa / atob</code> — Base64 编解码</li>
                                                    </ul>
                                                    <p className="font-semibold mb-1">Node.js 本地测试 shim（提交前删除）：</p>
                                                    <pre className="text-xs bg-muted rounded p-2 overflow-x-auto whitespace-pre">{`// --- Node.js test shim ---
var params = JSON.parse(process.argv[2] || '{}');
// node script.js '{"keyword":"iPhone","page":1}'
function getEnv(k) { return process.env[k] || ""; }
const fs = require('fs');
//finish run:  f.localSync("filename");
// --- end shim ---`}</pre>
                                                </PopoverContent>
                                            </Popover>
                                        </span>
                                    </FormLabel>
                                    <FormControl>
                                        <AsyncCodeEditor
                                            value={field.value}
                                            language="javascript"
                                            placeholder="请输入采集脚本代码..."
                                            onChange={(val) => field.onChange(val)}
                                            minHeight={350}
                                        />
                                    </FormControl>
                                    <FormDescription>
                                        编写数据采集脚本代码
                                    </FormDescription>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <div className="flex justify-end gap-2 pt-4">
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => onOpenChange(false)}
                                disabled={mutation.isPending}
                            >
                                取消
                            </Button>
                            <Button type="submit" disabled={mutation.isPending}>
                                {mutation.isPending && (
                                    <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />
                                )}
                                {isEdit ? "保存" : "确定"}
                            </Button>
                        </div>
                    </form>
                </Form>
            </DrawerContent>
        </Drawer>
    );
}
