import { UserSenderSmsHwConfigItemType, userSenderSmsHwConfigList } from "@shared/apis/user/sender-sms";
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
import { getQueryResponseData } from "@shared/lib/utils";
import { useQuery } from "@tanstack/react-query";
import { UseFormReturn } from "react-hook-form";

interface TplConfigDrawerHuaweiFormProps {
    form: UseFormReturn<any>;
}

export function TplConfigDrawerHuaweiForm({ form }: TplConfigDrawerHuaweiFormProps) {
    // 华为云配置列表
    const { data: configData } = useQuery({
        queryKey: ["hw-config-list"],
        queryFn: async ({ signal }) => {
            const result = await userSenderSmsHwConfigList({ ids: null }, { signal });
            return result;
        },
    });

    const configs = getQueryResponseData<UserSenderSmsHwConfigItemType[]>(configData, []);

    return (
        <>
            <FormField
                control={form.control}
                name="hw_config_id"
                render={({ field }) => (
                    <FormItem>
                        <FormLabel>华为云配置</FormLabel>
                        <Select
                            onValueChange={(value) => field.onChange(Number(value))}
                            value={field.value ? String(field.value) : ""}
                        >
                            <FormControl>
                                <SelectTrigger>
                                    <SelectValue placeholder="选择华为云配置" />
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
                name="signature"
                render={({ field }) => (
                    <FormItem>
                        <FormLabel>短信签名</FormLabel>
                        <FormControl>
                            <Input placeholder="您的短信签名" {...field} />
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
                        <FormLabel>发送者</FormLabel>
                        <FormControl>
                            <Input placeholder="发送者标识" {...field} />
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
                        <FormLabel>模板ID</FormLabel>
                        <FormControl>
                            <Input
                                type="number"
                                placeholder="123456"
                                {...field}
                                onChange={(e) => field.onChange(Number(e.target.value))}
                            />
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
                        <FormLabel>模板参数映射</FormLabel>
                        <FormControl>
                            <Input placeholder='{"code": "1"}' {...field} />
                        </FormControl>
                        <FormDescription>JSON格式的参数映射</FormDescription>
                        <FormMessage />
                    </FormItem>
                )}
            />
        </>
    );
}
