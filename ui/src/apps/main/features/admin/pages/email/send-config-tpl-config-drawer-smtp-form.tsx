import { SystemSenderMailerSmtpConfigItemType, systemSenderMailerSmtpConfigList, systemSenderMailerSmtpTplConfigAdd, SystemSenderMailerTplBodyItemType, systemSenderMailerTplBodyList } from "@shared/apis/admin/sender-mailer";
import { AutocompleteInput } from "@shared/components/custom/input/autocomplete-input";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading";
import { Button } from "@shared/components/ui/button";
import { Input } from "@shared/components/ui/input";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@shared/components/ui/select";
import { useToast } from "@shared/contexts/toast-context";
import { cn, formatServerError, getQueryResponseData } from "@shared/lib/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import { useDeferredValue, useState } from "react";
import { useForm } from "react-hook-form";
import {
    EmailChannelType,
    EmailSendConfigTplConfigSmtpFormSchema,
    type EmailSendConfigTplConfigSmtpFormType,
} from "./send-config-tpl-config-schema";

interface EmailSendConfigTplConfigDrawerSmtpFormProps {
    onClose: () => void;
}

// Inner component with Form provider and all fields
export function EmailSendConfigTplConfigDrawerSmtpForm({
    onClose,
}: EmailSendConfigTplConfigDrawerSmtpFormProps) {
    const toast = useToast();
    const queryClient = useQueryClient();

    // 搜索状态
    const [tplSearchMap, setTplSearchMap] = useState<{
        subject: string;
        body: string;
    }>({
        subject: "",
        body: "",
    });

    // 使用 useDeferredValue 延迟搜索值更新
    const deferredTplSearchMap = {
        subject: useDeferredValue(tplSearchMap.subject),
        body: useDeferredValue(tplSearchMap.body),
    };

    // SMTP配置列表
    const { data: smtpConfigData, isLoading, error, refetch } = useQuery({
        queryKey: ["admin-smtp-config-list"],
        queryFn: async ({ signal }) => {
            const result = await systemSenderMailerSmtpConfigList({}, { signal });
            return result;
        },
    });

    const smtpConfigs = getQueryResponseData<SystemSenderMailerSmtpConfigItemType[]>(smtpConfigData, []);

    // 主题模板列表
    const { data: subjectTplData, isLoading: isLoadingSubjectTpl } = useQuery({
        queryKey: ["admin-tpl-body-list", "subject", deferredTplSearchMap.subject],
        queryFn: async ({ signal }) => {
            const result = await systemSenderMailerTplBodyList({
                tpl_id_like: deferredTplSearchMap.subject || null,
                page: {
                    page: 1,
                    limit: 5,
                },
                count_num: false,
            }, { signal });
            return result;
        },
    });
    const subjectTplOptions = getQueryResponseData<SystemSenderMailerTplBodyItemType[]>(subjectTplData, []);

    // 正文模板列表
    const { data: bodyTplData, isLoading: isLoadingBodyTpl } = useQuery({
        queryKey: ["admin-tpl-body-list", "body", deferredTplSearchMap.body],
        queryFn: async ({ signal }) => {
            const result = await systemSenderMailerTplBodyList({
                tpl_id_like: deferredTplSearchMap.body || null,
                page: {
                    page: 1,
                    limit: 50,
                },
                count_num: false,
            }, { signal });
            return result;
        },
    });
    const bodyTplOptions = getQueryResponseData<SystemSenderMailerTplBodyItemType[]>(bodyTplData, []);

    const form = useForm<EmailSendConfigTplConfigSmtpFormType>({
        resolver: zodResolver(EmailSendConfigTplConfigSmtpFormSchema),
        defaultValues: {
            channel: EmailChannelType.SMTP,
            smtp_config_id: 0,
            name: "",
            tpl_key: "",
            from_email: "",
            reply_email: "",
            subject_tpl_id: "",
            body_tpl_id: "",
        },
    });

    const mutation = useMutation({
        mutationFn: (data: EmailSendConfigTplConfigSmtpFormType) =>
            systemSenderMailerSmtpTplConfigAdd({
                ...data,
                reply_email: data.reply_email || '',
            }),
        onSuccess: () => {
            toast.success("配置添加成功");
            queryClient.invalidateQueries({ queryKey: ["admin-tpl-config-list"] });
            form.reset();
            onClose();
        },
        onError: (error: any) => {
            toast.error(formatServerError(error));
        },
    });

    const onSubmit = (data: EmailSendConfigTplConfigSmtpFormType) => {
        mutation.mutate(data);
    };

    // 如果正在加载，显示加载状态
    if (isLoading) {
        return <CenteredLoading variant="content" iconSize="md" />;
    }

    // 如果加载失败，显示错误状态
    if (error) {
        return <CenteredError variant="content" error={error} onReset={refetch} />;
    }

    // Render form fields with error handling for context issues
    return (
        <div className="space-y-4">
            <div className="space-y-2">
                <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                    配置名称
                </label>
                <Input
                    placeholder="输入配置名称"
                    {...form.register("name")}
                />
                {form.formState.errors.name && (
                    <p className="text-sm font-medium text-destructive">
                        {form.formState.errors.name.message}
                    </p>
                )}
            </div>

            <div className="space-y-2">
                <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                    模板Key
                </label>
                <Input
                    placeholder="如: welcome_email"
                    {...form.register("tpl_key")}
                />
                <p className="text-sm text-muted-foreground">
                    唯一标识此邮件模板
                </p>
                {form.formState.errors.tpl_key && (
                    <p className="text-sm font-medium text-destructive">
                        {form.formState.errors.tpl_key.message}
                    </p>
                )}
            </div>

            <div className="space-y-2">
                <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                    SMTP配置
                </label>
                <Select
                    onValueChange={(value) => form.setValue("smtp_config_id", Number(value))}
                    value={form.watch("smtp_config_id") ? String(form.watch("smtp_config_id")) : ""}
                >
                    <SelectTrigger>
                        <SelectValue placeholder="选择SMTP配置" />
                    </SelectTrigger>
                    <SelectContent className="max-h-[300px]">
                        {smtpConfigs.map((config) => (
                            <SelectItem key={config.id} value={String(config.id)}>
                                {config.name} ({config.email})
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                <p className="text-sm text-muted-foreground">
                    选择用于发送邮件的SMTP配置
                </p>
                {form.formState.errors.smtp_config_id && (
                    <p className="text-sm font-medium text-destructive">
                        {form.formState.errors.smtp_config_id.message}
                    </p>
                )}
            </div>

            <div className="space-y-2">
                <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                    发件人邮箱
                </label>
                <Input
                    type="email"
                    placeholder="example@domain.com"
                    {...form.register("from_email")}
                />
                {form.formState.errors.from_email && (
                    <p className="text-sm font-medium text-destructive">
                        {form.formState.errors.from_email.message}
                    </p>
                )}
            </div>

            <div className="space-y-2">
                <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                    回复邮箱（可选）
                </label>
                <Input
                    type="email"
                    placeholder="reply@domain.com"
                    {...form.register("reply_email")}
                />
                <p className="text-sm text-muted-foreground">
                    收件人回复时的目标邮箱
                </p>
                {form.formState.errors.reply_email && (
                    <p className="text-sm font-medium text-destructive">
                        {form.formState.errors.reply_email.message}
                    </p>
                )}
            </div>

            <div className="space-y-2">
                <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                    主题模板ID
                </label>
                <AutocompleteInput
                    value={form.watch("subject_tpl_id")}
                    onChange={(value) => {
                        form.setValue("subject_tpl_id", value);
                        setTplSearchMap(prev => ({ ...prev, subject: value }));
                    }}
                    placeholder="输入或选择主题模板ID"
                    loading={isLoadingSubjectTpl}
                    options={subjectTplOptions.map(tpl => ({
                        value: tpl.tpl_id,
                        label: tpl.tpl_id
                    }))}
                    filterOnInput={true}
                />
                {form.formState.errors.subject_tpl_id && (
                    <p className="text-sm font-medium text-destructive">
                        {form.formState.errors.subject_tpl_id.message}
                    </p>
                )}
            </div>

            <div className="space-y-2">
                <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                    正文模板ID
                </label>
                <AutocompleteInput
                    value={form.watch("body_tpl_id")}
                    onChange={(value) => {
                        form.setValue("body_tpl_id", value);
                        setTplSearchMap(prev => ({ ...prev, body: value }));
                    }}
                    placeholder="输入或选择正文模板ID"
                    loading={isLoadingBodyTpl}
                    options={bodyTplOptions.map(tpl => ({
                        value: tpl.tpl_id,
                        label: tpl.tpl_id
                    }))}
                    filterOnInput={true}
                />
                {form.formState.errors.body_tpl_id && (
                    <p className="text-sm font-medium text-destructive">
                        {form.formState.errors.body_tpl_id.message}
                    </p>
                )}
            </div>

            <div className="flex justify-end gap-2 pt-4">
                <Button
                    type="button"
                    variant="outline"
                    onClick={onClose}
                    disabled={mutation.isPending}
                >
                    取消
                </Button>
                <Button
                    type="button"
                    disabled={mutation.isPending}
                    onClick={form.handleSubmit(onSubmit)}
                >
                    {mutation.isPending && (
                        <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />
                    )}
                    确定
                </Button>
            </div>
        </div>
    );
}

