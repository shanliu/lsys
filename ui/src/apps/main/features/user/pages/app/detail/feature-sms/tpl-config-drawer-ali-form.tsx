import { UserSenderSmsAliConfigItemType, userSenderSmsAliConfigList } from "@shared/apis/user/sender-sms";
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
import { getQueryResponseData } from "@shared/lib/utils";
import { useQuery } from "@tanstack/react-query";
import { UseFormReturn } from "react-hook-form";

interface TplConfigDrawerAliFormProps {
    form: UseFormReturn<any>;
}

export function TplConfigDrawerAliForm({ form }: TplConfigDrawerAliFormProps) {
    // 阿里云配置列表
    const { data: configData } = useQuery({
        queryKey: ["ali-config-list"],
        queryFn: async ({ signal }) => {
            const result = await userSenderSmsAliConfigList({ ids: null }, { signal });
            return result;
        },
    });

    const configs = getQueryResponseData<UserSenderSmsAliConfigItemType[]>(configData, []);

    return (
        <>
            <FormField
                control={form.control}
                name="ali_config_id"
                render={({ field }) => (
                    <FormItem>
                        <FormLabel>阿里云配置</FormLabel>
                        <Select
                            onValueChange={(value) => field.onChange(Number(value))}
                            value={field.value ? String(field.value) : ""}
                        >
                            <FormControl>
                                <SelectTrigger>
                                    <SelectValue placeholder="选择阿里云配置" />
                                </SelectTrigger>
                            </FormControl>
                            <SelectContent className="max-h-[300px]">
                                {configs.map((config) => (
                                    <SelectItem key={config.id} value={String(config.id)}>
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
                        <FormLabel>阿里云短信模板ID</FormLabel>
                        <FormControl>
                            <Input placeholder="SMS_123456789" {...field} />
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
                        <FormLabel>阿里云短信签名</FormLabel>
                        <FormControl>
                            <Input placeholder="您的短信签名" {...field} />
                        </FormControl>
                        <FormMessage />
                    </FormItem>
                )}
            />
        </>
    );
}
