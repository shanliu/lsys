import {
    systemSenderSmsConfigAdd,
    systemSenderSmsMapping,
} from "@shared/apis/admin/sender-sms";
import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { SenderRuleConfigForm } from "@apps/main/components/local/sender-config/rule-config-form";
import { NumberInput } from "@shared/components/custom/input/number-input";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
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
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@shared/components/ui/select";
import { useToast } from "@shared/contexts/toast-context";
import { cn, formatServerError } from "@shared/lib/utils";
import { DictItemType } from "@shared/types/apis-dict";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowUpDown, Loader2 } from "lucide-react";
import React from "react";
import { useForm } from "react-hook-form";
import {
    SmsSendConfigSendRuleFormSchema,
    type SmsSendConfigSendRuleFormType,
} from "./send-config-rule-schema";

interface SmsSendConfigRuleDrawerProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function SmsSendConfigRuleDrawer({
    open,
    onOpenChange,
}: SmsSendConfigRuleDrawerProps) {
    const toast = useToast();
    const queryClient = useQueryClient();

    const form = useForm<SmsSendConfigSendRuleFormType>({
        resolver: zodResolver(SmsSendConfigSendRuleFormSchema),
        defaultValues: {
            priority: 0,
            config_type: 0,
            config_data: undefined,
        },
    });

    const configType = form.watch("config_type");

    // 获取字典数据
    const {
        data: mappingData,
        isLoading: dictIsLoading,
        isError: dictError,
        error: dictErrors,
        refetch: refetchDict,
    } = useQuery({
        queryKey: ["admin-sender-sms-mapping"],
        queryFn: async ({ signal }) => {
            const result = await systemSenderSmsMapping({ signal });
            return result;
        },
    });

    const dictData = mappingData?.response || null;

    const mutation = useMutation({
        mutationFn: (data: SmsSendConfigSendRuleFormType) =>
            systemSenderSmsConfigAdd({
                priority: data.priority,
                config_type: data.config_type,
                config_data: data.config_data || 0,
            }),
        onSuccess: () => {
            toast.success("配置添加成功");
            queryClient.invalidateQueries({ queryKey: ["admin-sms-config-list"] });
            onOpenChange(false);
            form.reset();
        },
        onError: (error: any) => {
            toast.error(formatServerError(error));
        },
    });

    const onSubmit = (data: SmsSendConfigSendRuleFormType) => {
        mutation.mutate(data);
    };

    // 当配置类型改变时重置 config_data
    React.useEffect(() => {
        form.setValue("config_data", undefined);
    }, [configType, form]);

    // 如果字典加载失败，显示错误页面
    if (dictError && dictErrors) {
        return (
            <Drawer open={open} onOpenChange={onOpenChange}>
                <DrawerContent>
                    <CenteredError
                        variant="content"
                        error={dictErrors}
                        onReset={refetchDict}
                    />
                </DrawerContent>
            </Drawer>
        );
    }

    // 如果字典加载中，显示骨架屏
    if (dictIsLoading || !dictData) {
        return (
            <Drawer open={open} onOpenChange={onOpenChange}>
                <DrawerContent>
                    <PageSkeletonTable variant="content" />
                </DrawerContent>
            </Drawer>
        );
    }

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>新增短信发送规则</DrawerTitle>
                    <DrawerDescription>
                        填写短信发送规则配置信息
                    </DrawerDescription>
                </DrawerHeader>

                <Form {...form}>
                    <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 mt-6">
                        <FormField
                            control={form.control}
                            name="config_type"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>配置类型</FormLabel>
                                    <Select
                                        onValueChange={(value) => field.onChange(Number(value))}
                                        value={field.value ? String(field.value) : ""}
                                    >
                                        <FormControl>
                                            <SelectTrigger>
                                                <SelectValue placeholder="选择配置类型" />
                                            </SelectTrigger>
                                        </FormControl>
                                        <SelectContent className="max-h-[300px]">
                                            {dictData.sms_config_type?.map((item: DictItemType) => (
                                                <SelectItem key={item.key} value={item.key}>
                                                    {item.val}
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />
                        <SenderRuleConfigForm form={form} configType={configType} />
                        <FormField
                            control={form.control}
                            name="priority"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>优先级</FormLabel>
                                    <FormControl>
                                        <div className="flex items-center gap-2">
                                            <ArrowUpDown className="h-4 w-4 text-muted-foreground" />
                                            <NumberInput
                                                className="flex-1"
                                                min={0}
                                                max={100}
                                                placeholder="输入优先级"
                                                value={field.value}
                                                onChange={(value) => field.onChange(value)}
                                            />
                                        </div>
                                    </FormControl>
                                    <FormDescription>数值越大优先级越高</FormDescription>
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
                                确定
                            </Button>
                        </div>
                    </form>
                </Form>
            </DrawerContent>
        </Drawer>
    );
}
