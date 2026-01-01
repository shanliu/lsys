import { UserSenderSmsJdConfigItemType, userSenderSmsJdConfigList } from "@shared/apis/user/sender-sms";
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

interface TplConfigDrawerJdFormProps {
    form: UseFormReturn<any>;
}

export function TplConfigDrawerJdForm({ form }: TplConfigDrawerJdFormProps) {
    // 京东云配置列表
    const { data: configData } = useQuery({
        queryKey: ["jd-config-list"],
        queryFn: async ({ signal }) => {
            const result = await userSenderSmsJdConfigList({ ids: null }, { signal });
            return result;
        },
    });

    const configs = getQueryResponseData<UserSenderSmsJdConfigItemType[]>(configData, []);

    return (
        <>
            <FormField
                control={form.control}
                name="config_id"
                render={({ field }) => (
                    <FormItem>
                        <FormLabel>京东云配置</FormLabel>
                        <Select
                            onValueChange={(value) => field.onChange(Number(value))}
                            value={field.value ? String(field.value) : ""}
                        >
                            <FormControl>
                                <SelectTrigger>
                                    <SelectValue placeholder="选择京东云配置" />
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
                name="sign_id"
                render={({ field }) => (
                    <FormItem>
                        <FormLabel>签名ID</FormLabel>
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
                name="template_id"
                render={({ field }) => (
                    <FormItem>
                        <FormLabel>模板ID</FormLabel>
                        <FormControl>
                            <Input placeholder="tpl_123456" {...field} />
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
