import {
    userBarcodeCreateConfigAdd,
    userBarcodeCreateConfigEdit
} from "@shared/apis/user/barcode";
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
import { type TypedDictData } from "@apps/main/hooks/use-dict-data";
import { cn, formatServerError } from "@shared/lib/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import React from "react";
import { useForm } from "react-hook-form";
import {
    BarcodeCreateConfigFormSchema,
    type BarcodeCreateConfigFormType,
} from "./list-create-schema";

import { UserBarcodeCreateConfigItemType } from '@shared/apis/user/barcode';

interface BarcodeCreateConfigDrawerProps {
    appId: string | number;
    dictData: TypedDictData<["user_barcode"]>;
    config?: UserBarcodeCreateConfigItemType;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function BarcodeCreateConfigDrawer({
    appId,
    dictData,
    config,
    open,
    onOpenChange,
}: BarcodeCreateConfigDrawerProps) {
    const setOpen = onOpenChange;
    const toast = useToast();
    const queryClient = useQueryClient();
    const isEdit = !!config;

    const form = useForm<BarcodeCreateConfigFormType>({
        resolver: zodResolver(BarcodeCreateConfigFormSchema),
        defaultValues: config
            ? {
                barcode_type: config.barcode_type,
                status: Number(config.status),
                image_format: config.image_format,
                image_width: Number(config.image_width),
                image_height: Number(config.image_height),
                margin: Number(config.margin),
                image_color: config.image_color,
                image_background: config.image_background,
            }
            : {
                barcode_type: "",
                status: 1,
                image_format: "png",
                image_width: 300,
                image_height: 300,
                margin: 2,
                image_color: "#000000",
                image_background: "#FFFFFF",
            },
    });

    const mutation = useMutation({
        mutationFn: (data: BarcodeCreateConfigFormType) =>
            isEdit
                ? userBarcodeCreateConfigEdit({ id: config.id, ...data })
                : userBarcodeCreateConfigAdd({ app_id: Number(appId), ...data }),
        onSuccess: () => {
            toast.success(isEdit ? "配置更新成功" : "配置添加成功");
            queryClient.invalidateQueries({ queryKey: ["barcode-create-config-list", appId] });
            setOpen(false);
            if (!isEdit) {
                form.reset();
            }
        },
        onError: (error: any) => {
            toast.error(formatServerError(error));
        },
    });

    const onSubmit = (data: BarcodeCreateConfigFormType) => {
        mutation.mutate(data);
    };

    // Reset form when config changes or dialog opens
    React.useEffect(() => {
        if (open && config) {
            form.reset({
                barcode_type: config.barcode_type,
                status: Number(config.status),
                image_format: config.image_format,
                image_width: Number(config.image_width),
                image_height: Number(config.image_height),
                margin: Number(config.margin),
                image_color: config.image_color,
                image_background: config.image_background,
            });
        }
    }, [open, config, form]);

    return (
        <Drawer open={open} onOpenChange={setOpen}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>{isEdit ? "编辑条码配置" : "新增条码配置"}</DrawerTitle>
                    <DrawerDescription>
                        {isEdit ? "修改条码生成配置信息" : "填写条码生成配置信息"}
                    </DrawerDescription>
                </DrawerHeader>

                <Form {...form}>
                    <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 mt-6">
                        <FormField
                            control={form.control}
                            name="barcode_type"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>条码类型</FormLabel>
                                    <Select onValueChange={field.onChange} value={field.value}>
                                        <FormControl>
                                            <SelectTrigger>
                                                <SelectValue placeholder="选择条码类型" />
                                            </SelectTrigger>
                                        </FormControl>
                                        <SelectContent className="max-h-[300px]">
                                            {dictData.barcode_type?.map((item) => (
                                                <SelectItem key={item.key} value={item.key}>
                                                    {item.val}
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                    <FormDescription>二维码/条形码类型标识</FormDescription>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <FormField
                            control={form.control}
                            name="status"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>状态</FormLabel>
                                    <Select
                                        onValueChange={(value) => field.onChange(Number(value))}
                                        value={String(field.value)}
                                    >
                                        <FormControl>
                                            <SelectTrigger>
                                                <SelectValue placeholder="选择状态" />
                                            </SelectTrigger>
                                        </FormControl>
                                        <SelectContent className="max-h-[300px]">
                                            {dictData.create_status?.map((item) => (
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

                        <FormField
                            control={form.control}
                            name="image_format"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>图片格式</FormLabel>
                                    <Select onValueChange={field.onChange} value={field.value}>
                                        <FormControl>
                                            <SelectTrigger>
                                                <SelectValue placeholder="选择图片格式" />
                                            </SelectTrigger>
                                        </FormControl>
                                        <SelectContent className="max-h-[300px]">
                                            <SelectItem value="png">PNG</SelectItem>
                                            <SelectItem value="jpeg">JPEG</SelectItem>
                                            <SelectItem value="svg">SVG</SelectItem>
                                        </SelectContent>
                                    </Select>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <div className="grid grid-cols-2 gap-4">
                            <FormField
                                control={form.control}
                                name="image_width"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>图片宽度(px)</FormLabel>
                                        <FormControl>
                                            <Input
                                                type="number"
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
                                name="image_height"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>图片高度(px)</FormLabel>
                                        <FormControl>
                                            <Input
                                                type="number"
                                                {...field}
                                                onChange={(e) => field.onChange(Number(e.target.value))}
                                            />
                                        </FormControl>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />
                        </div>

                        <FormField
                            control={form.control}
                            name="margin"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>边距(px)</FormLabel>
                                    <FormControl>
                                        <Input
                                            type="number"
                                            {...field}
                                            onChange={(e) => field.onChange(Number(e.target.value))}
                                        />
                                    </FormControl>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <div className="grid grid-cols-2 gap-4">
                            <FormField
                                control={form.control}
                                name="image_color"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>前景色</FormLabel>
                                        <FormControl>
                                            <div className="flex gap-2">
                                                <Input type="color" {...field} className={cn("w-16 h-10")} />
                                                <Input {...field} placeholder="#000000" />
                                            </div>
                                        </FormControl>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />

                            <FormField
                                control={form.control}
                                name="image_background"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>背景色</FormLabel>
                                        <FormControl>
                                            <div className="flex gap-2">
                                                <Input type="color" {...field} className={cn("w-16 h-10")} />
                                                <Input {...field} placeholder="#FFFFFF" />
                                            </div>
                                        </FormControl>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />
                        </div>

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
                                {isEdit ? "保存" : "确定"}
                            </Button>
                        </div>
                    </form>
                </Form>
            </DrawerContent>
        </Drawer>
    );
}
