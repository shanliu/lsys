import {
    userSenderSmsAliAppConfigAdd,
    userSenderSmsCLoopenAppConfigAdd,
    userSenderSmsHwAppConfigAdd,
    userSenderSmsJdAppConfigAdd,
    userSenderSmsNeteaseAppConfigAdd,
    userSenderSmsTenAppConfigAdd,
} from "@shared/apis/user/sender-sms";
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
import { cn, formatServerError } from "@shared/lib/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import React from "react";
import { useForm } from "react-hook-form";
import { TplConfigDrawerAliForm } from "./tpl-config-drawer-ali-form";
import { TplConfigDrawerCloopenForm } from "./tpl-config-drawer-cloopen-form";
import { TplConfigDrawerHuaweiForm } from "./tpl-config-drawer-huawei-form";
import { TplConfigDrawerJdForm } from "./tpl-config-drawer-jd-form";
import { TplConfigDrawerNeteaseForm } from "./tpl-config-drawer-netease-form";
import { TplConfigDrawerTencentForm } from "./tpl-config-drawer-tencent-form";
import {
    SMS_PROVIDER_LABELS,
    SmsProviderType,
    TplConfigFormSchema,
    type TplConfigFormType,
} from "./tpl-config-schema";

interface TplConfigDrawerProps {
    appId: string | number;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function TplConfigDrawer({ appId, open, onOpenChange }: TplConfigDrawerProps) {
    const setOpen = onOpenChange;
    const [providerType, setProviderType] = React.useState<SmsProviderType | null>(null);
    const toast = useToast();
    const queryClient = useQueryClient();

    const form = useForm<TplConfigFormType>({
        resolver: zodResolver(TplConfigFormSchema),
    });

    const mutation = useMutation({
        mutationFn: async (data: TplConfigFormType) => {
            const appIdNum = Number(appId);

            switch (data.provider_type) {
                case SmsProviderType.ALI:
                    return userSenderSmsAliAppConfigAdd({
                        app_id: appIdNum,
                        name: data.name,
                        tpl_key: data.tpl_key,
                        ali_config_id: data.ali_config_id,
                        aliyun_sms_tpl: data.aliyun_sms_tpl,
                        aliyun_sign_name: data.aliyun_sign_name,
                    });
                case SmsProviderType.TENCENT:
                    return userSenderSmsTenAppConfigAdd({
                        app_id: appIdNum,
                        name: data.name,
                        tpl_key: data.tpl_key,
                        config_id: data.config_id,
                        sign_name: data.sign_name,
                        template_id: data.template_id,
                        template_map: data.template_map,
                    });
                case SmsProviderType.HUAWEI:
                    return userSenderSmsHwAppConfigAdd({
                        app_id: appIdNum,
                        name: data.name,
                        tpl_key: data.tpl_key,
                        hw_config_id: data.hw_config_id,
                        signature: data.signature,
                        sender: data.sender,
                        template_id: data.template_id,
                        template_map: data.template_map,
                    });
                case SmsProviderType.JD:
                    return userSenderSmsJdAppConfigAdd({
                        app_id: appIdNum,
                        name: data.name,
                        tpl_key: data.tpl_key,
                        config_id: data.config_id,
                        sign_id: data.sign_id,
                        template_id: data.template_id,
                        template_map: data.template_map,
                    });
                case SmsProviderType.NETEASE:
                    return userSenderSmsNeteaseAppConfigAdd({
                        app_id: appIdNum,
                        name: data.name,
                        tpl_key: data.tpl_key,
                        config_id: data.config_id,
                        template_id: data.template_id,
                        template_map: data.template_map,
                    });
                case SmsProviderType.CLOOPEN:
                    return userSenderSmsCLoopenAppConfigAdd({
                        app_id: appIdNum,
                        name: data.name,
                        tpl_key: data.tpl_key,
                        config_id: data.config_id,
                        template_id: data.template_id,
                        template_map: data.template_map,
                    });
                default:
                    throw new Error("不支持的提供商类型");
            }
        },
        onSuccess: () => {
            toast.success("配置添加成功");
            queryClient.invalidateQueries({ queryKey: ["tpl-config-list", appId] });
            setOpen(false);
            form.reset();
            setProviderType(null);
        },
        onError: (error: any) => {
            toast.error(formatServerError(error));
        },
    });

    const onSubmit = (data: TplConfigFormType) => {
        mutation.mutate(data);
    };

    // 当提供商类型改变时，重置表单
    const handleProviderTypeChange = (type: SmsProviderType) => {
        setProviderType(type);
        form.reset({ provider_type: type } as any);
    };

    // 根据提供商类型渲染对应的表单组件
    const renderProviderForm = () => {
        switch (providerType) {
            case SmsProviderType.ALI:
                return <TplConfigDrawerAliForm form={form} />;
            case SmsProviderType.TENCENT:
                return <TplConfigDrawerTencentForm form={form} />;
            case SmsProviderType.HUAWEI:
                return <TplConfigDrawerHuaweiForm form={form} />;
            case SmsProviderType.JD:
                return <TplConfigDrawerJdForm form={form} />;
            case SmsProviderType.NETEASE:
                return <TplConfigDrawerNeteaseForm form={form} />;
            case SmsProviderType.CLOOPEN:
                return <TplConfigDrawerCloopenForm form={form} />;
            default:
                return null;
        }
    };

    return (
        <Drawer open={open} onOpenChange={setOpen}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>新增短信模板配置</DrawerTitle>
                    <DrawerDescription>选择短信服务商并填写配置信息</DrawerDescription>
                </DrawerHeader>

                <Form {...form}>
                    <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 mt-6">
                        {/* 提供商类型选择 */}
                        <div className="space-y-2">
                            <label className="text-sm font-medium">短信服务提供商</label>
                            <Select
                                value={providerType || ""}
                                onValueChange={(value) => handleProviderTypeChange(value as SmsProviderType)}
                            >
                                <SelectTrigger>
                                    <SelectValue placeholder="选择短信服务提供商" />
                                </SelectTrigger>
                                <SelectContent className="max-h-[300px]">
                                    {Object.entries(SMS_PROVIDER_LABELS).map(([key, label]) => (
                                        <SelectItem key={key} value={key}>
                                            {label}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        {providerType && (
                            <>
                                {/* 通用字段 */}
                                <FormField
                                    control={form.control}
                                    name="name"
                                    render={({ field }) => (
                                        <FormItem>
                                            <FormLabel>配置名称</FormLabel>
                                            <FormControl>
                                                <Input placeholder="输入配置名称" {...field} />
                                            </FormControl>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />

                                <FormField
                                    control={form.control}
                                    name="tpl_key"
                                    render={({ field }) => (
                                        <FormItem>
                                            <FormLabel>模板Key</FormLabel>
                                            <FormControl>
                                                <Input placeholder="如: verify_code" {...field} />
                                            </FormControl>
                                            <FormDescription>唯一标识此短信模板</FormDescription>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />

                                {/* 根据提供商类型渲染对应的表单 */}
                                {renderProviderForm()}

                                <div className="flex justify-end gap-2 pt-4">
                                    <Button
                                        type="button"
                                        variant="outline"
                                        onClick={() => setOpen(false)}
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
                            </>
                        )}
                    </form>
                </Form>
            </DrawerContent>
        </Drawer>
    );
}
