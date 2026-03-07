import { EmaySmsConfigItemType, emaySmsConfigList } from "@shared/apis/admin/sender-sms";
import {
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
import { cn, getQueryResponseData } from "@shared/lib/utils";
import { useQuery } from "@tanstack/react-query";
import { UseFormReturn } from "react-hook-form";
import { z } from "zod";
import { EmaySmsConfigTplFormSchema, SmsProviderType } from "./send-config-tpl-config-schema";

type EmayFormType = z.infer<typeof EmaySmsConfigTplFormSchema>;

interface SmsSendConfigTplConfigDrawerEmayFormProps {
    form: UseFormReturn<EmayFormType>;
}

export function SmsSendConfigTplConfigDrawerEmayForm({
    form,
}: SmsSendConfigTplConfigDrawerEmayFormProps) {
    // 获取亿美软通配置列表
    const { data: configData } = useQuery({
        queryKey: ["emay-sms-config-list"],
        queryFn: async () => {
            const result = await emaySmsConfigList({});
            return result;
        },
    });

    const configs = getQueryResponseData<EmaySmsConfigItemType[]>(configData, []);

    return (
        <>
            <input
                type="hidden"
                {...form.register("provider_type")}
                value={SmsProviderType.EMAY}
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
                name="config_id"
                render={({ field }) => (
                    <FormItem>
                        <FormLabel>亿美软通配置 <span className={cn("text-red-500")}>*</span></FormLabel>
                        <Select
                            onValueChange={(value) => field.onChange(Number(value))}
                            value={field.value?.toString()}
                        >
                            <FormControl>
                                <SelectTrigger>
                                    <SelectValue placeholder="请选择亿美软通配置" />
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
                name="extended_code"
                render={({ field }) => (
                    <FormItem>
                        <FormLabel>扩展码</FormLabel>
                        <FormControl>
                            <Input placeholder="选填，扩展码" {...field} />
                        </FormControl>
                        <FormDescription>短信扩展码，可为空</FormDescription>
                        <FormMessage />
                    </FormItem>
                )}
            />
        </>
    );
}
