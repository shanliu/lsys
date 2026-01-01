import {
    aliSmsTplConfigAdd,
    cloopenSmsTplConfigAdd,
    hwSmsTplConfigAdd,
    jdSmsTplConfigAdd,
    neteaseSmsTplConfigAdd,
    tencentSmsTplConfigAdd,
} from "@shared/apis/admin/sender-sms";
import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { Button } from "@shared/components/ui/button";
import { Form } from "@shared/components/ui/form";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@shared/components/ui/select";
import { useToast } from "@shared/contexts/toast-context";
import { cn, formatServerError } from "@shared/lib/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import React from "react";
import { useForm } from "react-hook-form";
import { SmsSendConfigTplConfigDrawerAliForm } from "./send-config-tpl-config-drawer-ali-form";
import { SmsSendConfigTplConfigDrawerCloopenForm } from "./send-config-tpl-config-drawer-cloopen-form";
import { SmsSendConfigTplConfigDrawerHuaweiForm } from "./send-config-tpl-config-drawer-huawei-form";
import { SmsSendConfigTplConfigDrawerJdForm } from "./send-config-tpl-config-drawer-jd-form";
import { SmsSendConfigTplConfigDrawerNeteaseForm } from "./send-config-tpl-config-drawer-netease-form";
import { SmsSendConfigTplConfigDrawerTencentForm } from "./send-config-tpl-config-drawer-tencent-form";
import {
    SMS_PROVIDER_LABELS,
    SmsProviderType,
    TplConfigFormSchema,
    type TplConfigFormType,
} from "./send-config-tpl-config-schema";

interface SmsSendConfigTplConfigDrawerProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function SmsSendConfigTplConfigDrawer({ open, onOpenChange }: SmsSendConfigTplConfigDrawerProps) {
    const setOpen = onOpenChange;
    const [providerType, setProviderType] = React.useState<SmsProviderType | null>(null);
    const toast = useToast();
    const queryClient = useQueryClient();

    const form = useForm<TplConfigFormType>({
        resolver: zodResolver(TplConfigFormSchema),
    });

    const mutation = useMutation({
        mutationFn: async (data: TplConfigFormType) => {
            switch (data.provider_type) {
                case SmsProviderType.ALI:
                    return aliSmsTplConfigAdd({
                        name: data.name,
                        tpl_key: data.tpl_key,
                        ali_config_id: data.ali_config_id,
                        aliyun_sms_tpl: data.aliyun_sms_tpl,
                        aliyun_sign_name: data.aliyun_sign_name,
                    });
                case SmsProviderType.TENCENT:
                    return tencentSmsTplConfigAdd({
                        name: data.name,
                        tpl_key: data.tpl_key,
                        config_id: data.config_id,
                        sign_name: data.sign_name,
                        template_id: data.template_id,
                        template_map: data.template_map,
                    });
                case SmsProviderType.HUAWEI:
                    return hwSmsTplConfigAdd({
                        name: data.name,
                        tpl_key: data.tpl_key,
                        hw_config_id: data.hw_config_id,
                        signature: data.signature,
                        sender: data.sender,
                        template_id: data.template_id,
                        template_map: data.template_map,
                    });
                case SmsProviderType.JD:
                    return jdSmsTplConfigAdd({
                        name: data.name,
                        tpl_key: data.tpl_key,
                        config_id: data.config_id,
                        sign_id: data.sign_id,
                        template_id: data.template_id,
                        template_map: data.template_map,
                    });
                case SmsProviderType.NETEASE:
                    return neteaseSmsTplConfigAdd({
                        name: data.name,
                        tpl_key: data.tpl_key,
                        config_id: data.config_id,
                        template_id: data.template_id,
                        template_map: data.template_map,
                    });
                case SmsProviderType.CLOOPEN:
                    return cloopenSmsTplConfigAdd({
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
            queryClient.invalidateQueries({ queryKey: ["admin-tpl-config-list"] });
            setOpen(false);
            form.reset();
            setProviderType(null);
        },
        onError: (error: any) => {
            toast.error(formatServerError(error, "添加失败"));
        },
    });

    const onSubmit = (data: TplConfigFormType) => {
        mutation.mutate(data);
    };

    const handleOpenChange = (open: boolean) => {
        if (!open) {
            form.reset();
            setProviderType(null);
        }
        setOpen(open);
    };

    return (
        <Drawer open={open} onOpenChange={handleOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>新增短信模板配置</DrawerTitle>
                    <DrawerDescription>
                        选择短信服务提供商并填写相关配置信息
                    </DrawerDescription>
                </DrawerHeader>

                <div className={cn("mt-6 space-y-6")}>
                    <div className={cn("space-y-2")}>
                        <label className={cn("text-sm font-medium")}>
                            选择服务提供商 <span className={cn("text-red-500")}>*</span>
                        </label>
                        <Select
                            value={providerType || ""}
                            onValueChange={(value) => {
                                setProviderType(value as SmsProviderType);
                                form.reset();
                            }}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder="请选择服务提供商" />
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
                        <Form {...form}>
                            <form onSubmit={form.handleSubmit(onSubmit)} className={cn("space-y-4")}>
                                {providerType === SmsProviderType.ALI && (
                                    <SmsSendConfigTplConfigDrawerAliForm form={form as any} />
                                )}
                                {providerType === SmsProviderType.TENCENT && (
                                    <SmsSendConfigTplConfigDrawerTencentForm form={form as any} />
                                )}
                                {providerType === SmsProviderType.HUAWEI && (
                                    <SmsSendConfigTplConfigDrawerHuaweiForm form={form as any} />
                                )}
                                {providerType === SmsProviderType.JD && (
                                    <SmsSendConfigTplConfigDrawerJdForm form={form as any} />
                                )}
                                {providerType === SmsProviderType.NETEASE && (
                                    <SmsSendConfigTplConfigDrawerNeteaseForm form={form as any} />
                                )}
                                {providerType === SmsProviderType.CLOOPEN && (
                                    <SmsSendConfigTplConfigDrawerCloopenForm form={form as any} />
                                )}

                                <div className={cn("flex justify-end gap-2 pt-4")}>
                                    <Button type="submit" disabled={mutation.isPending}>
                                        {mutation.isPending && <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />}
                                        保存
                                    </Button>
                                </div>
                            </form>
                        </Form>
                    )}
                </div>
            </DrawerContent>
        </Drawer>
    );
}
