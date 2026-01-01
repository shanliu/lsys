import { aliSmsConfigList, AliSmsConfigItemType } from "@shared/apis/admin/sender-sms";
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
import { AliSmsConfigFormSchema, SmsProviderType } from "./send-config-tpl-config-schema";
import { z } from "zod";

type AliFormType = z.infer<typeof AliSmsConfigFormSchema>;

interface SmsSendConfigTplConfigDrawerAliFormProps {
    form: UseFormReturn<AliFormType>;
}

export function SmsSendConfigTplConfigDrawerAliForm({
    form,
}: SmsSendConfigTplConfigDrawerAliFormProps) {
    // 获取阿里云配置列表
    const { data: configData } = useQuery({
        queryKey: ["ali-sms-config-list"],
        queryFn: async () => {
            const result = await aliSmsConfigList({});
            return result;
        },
    });

    const configs = getQueryResponseData<AliSmsConfigItemType[]>(configData, []);

    return (
        <>
            <input
                type="hidden"
                {...form.register("provider_type")}
                value={SmsProviderType.ALI}
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
                    name="ali_config_id"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>阿里云配置 <span className={cn("text-red-500")}>*</span></FormLabel>
                            <Select
                                onValueChange={(value) => field.onChange(Number(value))}
                                value={field.value?.toString()}
                            >
                                <FormControl>
                                    <SelectTrigger>
                                        <SelectValue placeholder="请选择阿里云配置" />
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
                    name="aliyun_sms_tpl"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>阿里云短信模板ID <span className={cn("text-red-500")}>*</span></FormLabel>
                            <FormControl>
                                <Input placeholder="请输入阿里云短信模板ID" {...field} />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                <FormField
                    control={form.control}
                    name="aliyun_sign_name"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>阿里云短信签名 <span className={cn("text-red-500")}>*</span></FormLabel>
                            <FormControl>
                                <Input placeholder="请输入阿里云短信签名" {...field} />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />
        </>
    );
}
