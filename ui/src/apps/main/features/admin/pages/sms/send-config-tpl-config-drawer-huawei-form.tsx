import { hwSmsConfigList, HwSmsConfigItemType } from "@shared/apis/admin/sender-sms";
import {
    FormControl,
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
import { cn, getQueryResponseData } from "@shared/lib/utils";
import { useQuery } from "@tanstack/react-query";
import { UseFormReturn } from "react-hook-form";
import { HuaweiSmsConfigFormSchema, SmsProviderType } from "./send-config-tpl-config-schema";
import { z } from "zod";

type HuaweiFormType = z.infer<typeof HuaweiSmsConfigFormSchema>;

interface SmsSendConfigTplConfigDrawerHuaweiFormProps {
    form: UseFormReturn<HuaweiFormType>;
}

export function SmsSendConfigTplConfigDrawerHuaweiForm({
    form,
}: SmsSendConfigTplConfigDrawerHuaweiFormProps) {
    // 获取华为云配置列表
    const { data: configData } = useQuery({
        queryKey: ["hw-sms-config-list"],
        queryFn: async () => {
            const result = await hwSmsConfigList({});
            return result;
        },
    });

    const configs = getQueryResponseData<HwSmsConfigItemType[]>(configData, []);

    return (
        <>
            <input
                type="hidden"
                {...form.register("provider_type")}
                value={SmsProviderType.HUAWEI}
            />

                <FormField
                    control={form.control}
                    name="name"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>配置名称 <span className={cn("text-red-500")}>*</span></FormLabel>
                            <FormControl>
                                <Input placeholder="请输入配置名称" {...field} />
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
                            <FormLabel>模板Key <span className={cn("text-red-500")}>*</span></FormLabel>
                            <FormControl>
                                <Input placeholder="请输入模板Key" {...field} />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                <FormField
                    control={form.control}
                    name="hw_config_id"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>华为云配置 <span className={cn("text-red-500")}>*</span></FormLabel>
                            <Select
                                onValueChange={(value) => field.onChange(Number(value))}
                                value={field.value?.toString()}
                            >
                                <FormControl>
                                    <SelectTrigger>
                                        <SelectValue placeholder="请选择华为云配置" />
                                    </SelectTrigger>
                                </FormControl>
                                <SelectContent className="max-h-[300px]">
                                    {configs?.map((config: any) => (
                                        <SelectItem key={config.id} value={config.id.toString()}>
                                            {config.name}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                <FormField
                    control={form.control}
                    name="signature"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>短信签名 <span className={cn("text-red-500")}>*</span></FormLabel>
                            <FormControl>
                                <Input placeholder="请输入短信签名" {...field} />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                <FormField
                    control={form.control}
                    name="sender"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>发送者 <span className={cn("text-red-500")}>*</span></FormLabel>
                            <FormControl>
                                <Input placeholder="请输入发送者" {...field} />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                <FormField
                    control={form.control}
                    name="template_id"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>模板ID <span className={cn("text-red-500")}>*</span></FormLabel>
                            <FormControl>
                                <Input placeholder="请输入模板ID" {...field} />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                <FormField
                    control={form.control}
                    name="template_map"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>模板参数映射 <span className={cn("text-red-500")}>*</span></FormLabel>
                            <FormControl>
                                <Input placeholder="请输入模板参数映射，例如: {code:code,time:time}" {...field} />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />
        </>
    );
}
