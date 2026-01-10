import { UserSenderSmsCLoopenConfigItemType, userSenderSmsCLoopenConfigList } from "@shared/apis/user/sender-sms";
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

interface TplConfigDrawerCloopenFormProps {
    form: UseFormReturn<any>;
}

export function TplConfigDrawerCloopenForm({ form }: TplConfigDrawerCloopenFormProps) {
    // 容联云配置列表
    const { data: configData } = useQuery({
        queryKey: ["cloopen-config-list"],
        queryFn: async ({ signal }) => {
            const result = await userSenderSmsCLoopenConfigList({ ids: null }, { signal });
            return result;
        },
    });

    const configs = getQueryResponseData<UserSenderSmsCLoopenConfigItemType[]>(configData, []);

    return (
        <>
            <FormField
                control={form.control}
                name="config_id"
                render={({ field }) => (
                    <FormItem>
                        <FormLabel>容联云配置</FormLabel>
                        <Select
                            onValueChange={(value) => field.onChange(Number(value))}
                            value={field.value ? String(field.value) : ""}
                        >
                            <FormControl>
                                <SelectTrigger>
                                    <SelectValue placeholder="选择容联云配置" />
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
                name="template_id"
                render={({ field }) => (
                    <FormItem>
                        <FormLabel>容联云短信模板ID</FormLabel>
                        <FormControl>
                            <Input
                                placeholder="如: tpl-001"
                                {...field}
                                value={field.value ?? ""}
                                onChange={(e) => field.onChange(e.target.value)}
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
