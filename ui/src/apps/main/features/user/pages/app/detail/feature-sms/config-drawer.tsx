import {
    userSenderSmsConfigAdd,
} from "@shared/apis/user/sender-sms";
import { SenderRuleConfigForm } from "@apps/main/components/local/sender-config/rule-config-form";
import { NumberInput } from "@shared/components/custom/input/number-input";
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
import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { useToast } from "@shared/contexts/toast-context";
import { type TypedDictData } from "@apps/main/hooks/use-dict-data";
import { cn, formatServerError } from "@shared/lib/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import React from "react";
import { useForm } from "react-hook-form";
import {
    SmsConfigFormSchema,
    type SmsConfigFormType,
} from "./config-schema";

interface SmsConfigDrawerProps {
    appId: string | number;
    dictData: TypedDictData<["user_sender_sms"]>;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function SmsConfigDrawer({
    appId,
    dictData,
    open,
    onOpenChange,
}: SmsConfigDrawerProps) {
    const toast = useToast();
    const queryClient = useQueryClient();

    const form = useForm<SmsConfigFormType>({
        resolver: zodResolver(SmsConfigFormSchema),
        defaultValues: {
            priority: 0,
            config_type: 0,
            config_data: undefined,
        },
    });

    const configType = form.watch("config_type");

    const mutation = useMutation({
        mutationFn: (data: SmsConfigFormType) =>
            userSenderSmsConfigAdd({ app_id: Number(appId), ...data }),
        onSuccess: () => {
            toast.success("配置添加成功");
            queryClient.invalidateQueries({ queryKey: ["sms-config-list", appId] });
            onOpenChange(false);
            form.reset();
        },
        onError: (error: any) => {
            toast.error(formatServerError(error));
        },
    });

    const onSubmit = (data: SmsConfigFormType) => {
        mutation.mutate(data);
    };

    // 当配置类型改变时重置 config_data
    React.useEffect(() => {
        if (configType === 1) {
            form.setValue("config_data", '');
        } else if (configType === 2) {
            form.setValue("config_data", { range_time: 0, max_send: 0 });
        } else if (configType === 3) {
            form.setValue("config_data", 0);
        } else if (configType === 4 || configType === 20 || configType === 21) {
            form.setValue("config_data", '');
        }
    }, [configType, form]);

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>新增短信发送配置</DrawerTitle>
                    <DrawerDescription>
                        填写短信发送配置信息
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
                                            {dictData.sms_config_type?.map((item) => (
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
                                        <NumberInput
                                            min={0}
                                            max={100}
                                            placeholder="输入优先级"
                                            value={field.value}
                                            onChange={(value) => field.onChange(value)}
                                        />
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
