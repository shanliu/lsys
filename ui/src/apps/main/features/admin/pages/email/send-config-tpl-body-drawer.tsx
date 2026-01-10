import {
    systemSenderMailerTplBodyAdd,
    systemSenderMailerTplBodyEdit,
    type SystemSenderMailerTplBodyItemType
} from "@shared/apis/admin/sender-mailer";
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
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle
} from "@apps/main/components/local/drawer";
import { useToast } from "@shared/contexts/toast-context";
import { cn, formatServerError } from "@shared/lib/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import CodeEditor from "@uiw/react-textarea-code-editor";
import { Loader2 } from "lucide-react";
import React from "react";
import { useForm } from "react-hook-form";
import { EmailSendConfigTplBodyFormSchema, EmailSendConfigTplBodyFormType } from "./send-config-tpl-body-schema";

interface EmailSendConfigTplBodyDrawerProps {
    tpl?: SystemSenderMailerTplBodyItemType;
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSuccess?: () => void;
}

export function EmailSendConfigTplBodyDrawer({ tpl, open, onOpenChange, onSuccess }: EmailSendConfigTplBodyDrawerProps) {
    const toast = useToast();
    const queryClient = useQueryClient();
    const isEdit = !!tpl;

    const form = useForm<EmailSendConfigTplBodyFormType>({
        resolver: zodResolver(EmailSendConfigTplBodyFormSchema),
        defaultValues: tpl
            ? {
                tpl_id: tpl.tpl_id,
                tpl_data: tpl.tpl_data,
            }
            : {
                tpl_id: "",
                tpl_data: "",
            },
    });

    const mutation = useMutation({
        mutationFn: (data: EmailSendConfigTplBodyFormType) =>
            isEdit
                ? systemSenderMailerTplBodyEdit({ id: tpl.id, tpl_data: data.tpl_data })
                : systemSenderMailerTplBodyAdd(data),
        onSuccess: () => {
            toast.success(isEdit ? "模板更新成功" : "模板添加成功");
            queryClient.invalidateQueries({ queryKey: ["admin-mail-tpl-list"] });
            onOpenChange(false);
            onSuccess?.();
            if (!isEdit) {
                form.reset();
            }
        },
        onError: (error: any) => {
            toast.error(formatServerError(error));
        },
    });

    const onSubmit = (data: EmailSendConfigTplBodyFormType) => {
        mutation.mutate(data);
    };

    // Reset form when tpl changes or dialog opens
    React.useEffect(() => {
        if (open && tpl) {
            form.reset({
                tpl_id: tpl.tpl_id,
                tpl_data: tpl.tpl_data,
            });
        }
    }, [open, tpl, form]);

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>{isEdit ? "编辑邮件模板" : "新增邮件模板"}</DrawerTitle>
                    <DrawerDescription>
                        {isEdit ? "修改邮件模板内容" : "填写邮件模板信息"}
                    </DrawerDescription>
                </DrawerHeader>

                <Form {...form}>
                    <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 mt-6">
                        <FormField
                            control={form.control}
                            name="tpl_id"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>模板ID</FormLabel>
                                    <FormControl>
                                        <Input
                                            placeholder="如: welcome_email"
                                            {...field}
                                            disabled={isEdit}
                                        />
                                    </FormControl>
                                    <FormDescription>
                                        {isEdit ? "模板ID不可修改" : "用于标识模板的唯一ID"}
                                    </FormDescription>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <FormField
                            control={form.control}
                            name="tpl_data"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>模板内容</FormLabel>
                                    <FormControl>
                                        <div className="border rounded-md overflow-hidden bg-muted/30">
                                            <CodeEditor
                                                value={field.value}
                                                language="html"
                                                placeholder="请输入模板内容，支持HTML格式"
                                                onChange={(evn) => field.onChange(evn.target.value)}
                                                padding={15}
                                                minHeight={400}
                                                style={{
                                                    fontSize: 13,
                                                    backgroundColor: "hsl(var(--muted) / 0.3)",
                                                    fontFamily:
                                                        "ui-monospace,SFMono-Regular,SF Mono,Consolas,Liberation Mono,Menlo,monospace",
                                                    lineHeight: "1.5",
                                                }}
                                                className="w-full"
                                            />
                                        </div>
                                    </FormControl>
                                    <FormDescription>
                                        支持HTML格式，可使用变量如 {"{"}{"{"} name {"}"}{"}"}
                                    </FormDescription>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <div className="flex justify-end gap-2 pt-4">
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => onOpenChange(false)}
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
