import {
    userSenderMailerMessageSend,
    userSenderMailerTplConfigList,
    type UserSenderMailerTplConfigItemType
} from "@shared/apis/user/sender-mailer"
import { AutocompleteInput } from "@shared/components/custom/input/autocomplete-input"
import { DateTimePicker } from "@shared/components/custom/input/datetime-picker"
import { NumberInput } from "@shared/components/custom/input/number-input"
import { Button } from "@shared/components/ui/button"
import {
    Form,
    FormControl,
    FormDescription,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
} from "@shared/components/ui/form"
import { Input } from "@shared/components/ui/input"
import { useToast } from "@shared/contexts/toast-context"
import { cn, formatServerError, getQueryResponseData } from "@shared/lib/utils"
import { EmailSchema } from "@shared/types/base-schema"
import { zodResolver } from "@hookform/resolvers/zod"
import { useMutation, useQuery } from "@tanstack/react-query"
import CodeEditor from "@uiw/react-textarea-code-editor"
import { format } from "date-fns"
import { Delimiter, Tag, TagInput } from "emblor"
import { Loader2 } from "lucide-react"
import React, { useEffect, useState } from "react"
import { useForm } from "react-hook-form"
import { MailSendFormSchema, type MailSendFormType } from "./send-schema"

// 自定义防抖 hook
function useDebounce<T>(value: T, delay: number): T {
    const [debouncedValue, setDebouncedValue] = useState<T>(value)

    useEffect(() => {
        const handler = setTimeout(() => {
            setDebouncedValue(value)
        }, delay)

        return () => {
            clearTimeout(handler)
        }
    }, [value, delay])

    return debouncedValue
}

interface AppDetailFeatureMailSendFormProps {
    appId: number
}

export function AppDetailFeatureMailSendForm({ appId }: AppDetailFeatureMailSendFormProps) {
    const toast = useToast()

    // 模板搜索关键词
    const [tplSearchKeyword, setTplSearchKeyword] = useState("")
    const debouncedTplSearch = useDebounce(tplSearchKeyword, 500)

    // 收件人标签状态
    const [emailTags, setEmailTags] = useState<Tag[]>([])
    const [activeTagIndex, setActiveTagIndex] = useState<number | null>(null)

    const form = useForm<MailSendFormType>({
        resolver: zodResolver(MailSendFormSchema),
        defaultValues: {
            tpl_key: "",
            to: [],
            data: "{}",
            reply: "",
            send_time: undefined,
            max_try: 0,
        },
    })

    // 加载模板列表
    const {
        data: tplConfigData,
        isLoading: isLoadingTplConfig,
    } = useQuery({
        queryKey: ["mail-tpl-config-list", appId, debouncedTplSearch || ""],
        queryFn: ({ signal }) => userSenderMailerTplConfigList({
            app_id: appId,
            like_tpl: debouncedTplSearch.trim() || null,
            page: { page: 1, limit: 50 },
            count_num: false,
        }, { signal }),
        enabled: !!appId,
        staleTime: 2 * 60 * 1000,
    })

    const tplConfigs = getQueryResponseData<UserSenderMailerTplConfigItemType[]>(tplConfigData, [])

    // 将模板转换为下拉选项
    const tplOptions = React.useMemo(() => {
        return tplConfigs.map((tpl) => {
            const details = [
                tpl.setting_name && `渠道: ${tpl.setting_name}`,
                tpl.config_data?.from_email && `发件人: ${tpl.config_data.from_email}`,
            ].filter(Boolean).join(' | ')
            return {
                value: tpl.tpl_key,
                label: `${tpl.name} (${tpl.tpl_key})${details ? ` - ${details}` : ''}`,
            }
        })
    }, [tplConfigs])

    // 发送邮件
    const sendMutation = useMutation({
        mutationFn: (data: MailSendFormType) => {
            const params: any = {
                app_id: appId,
                tpl_key: data.tpl_key,
                to: data.to,
                data: data.data.trim() ? JSON.parse(data.data) : {},
            }

            if (data.reply && data.reply.trim()) {
                params.reply = data.reply
            }

            if (data.send_time) {
                params.send_time = format(data.send_time, 'yyyy-MM-dd HH:mm:ss')
            }

            if (data.max_try && data.max_try > 0) {
                params.max_try = data.max_try
            }

            return userSenderMailerMessageSend(params)
        },
        onSuccess: (result) => {
            if (result.status) {
                toast.success("邮件发送成功")
                form.reset()
                setEmailTags([])
                setTplSearchKeyword("")
            } else {
                toast.error(formatServerError(result))
            }
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        },
    })

    const onSubmit = (data: MailSendFormType) => {
        sendMutation.mutate(data)
    }

    // 验证邮箱标签
    const validateEmailTag = (tag: string): boolean => {
        return EmailSchema.safeParse(tag.trim()).success
    }

    // 处理标签变化
    const handleTagsChange = (newTags: Tag[]) => {
        setEmailTags(newTags)
        form.setValue("to", newTags.map(t => t.text), { shouldValidate: true })
    }

    // 最小可选时间为当前时间
    const minDateTime = new Date()

    // 阻止 Enter 键提交表单（允许在特定组件中使用 Enter 键）
    const handleFormKeyDown = (e: React.KeyboardEvent<HTMLFormElement>) => {
        if (e.key === "Enter") {
            const target = e.target as HTMLElement
            // 如果是提交按钮或多行文本框则允许
            if (target.tagName === "BUTTON" || target.tagName === "TEXTAREA") {
                return
            }
            // 否则阻止表单提交
            e.preventDefault()
        }
    }

    return (
        <Form {...form}>
            <form
                onSubmit={form.handleSubmit(onSubmit)}
                onKeyDown={handleFormKeyDown}
                className={cn("space-y-6")}
            >
                {/* 收件人 - 第一行 */}
                <FormField
                    control={form.control}
                    name="to"
                    render={() => (
                        <FormItem>
                            <FormLabel>收件人 *</FormLabel>
                            <FormControl>
                                <TagInput
                                    tags={emailTags}
                                    setTags={handleTagsChange as any}
                                    placeholder="输入邮箱后按回车添加"
                                    validateTag={validateEmailTag}
                                    activeTagIndex={activeTagIndex}
                                    setActiveTagIndex={setActiveTagIndex}
                                    styleClasses={{
                                        input: "bg-transparent border-0 shadow-none focus-visible:ring-0 focus-visible:ring-offset-0 outline-none p-0",
                                        inlineTagsContainer: "border rounded-md p-2 gap-2 min-h-10",
                                        tag: {
                                            body: "bg-secondary text-secondary-foreground",
                                            closeButton: "hover:bg-destructive/20",
                                        },
                                    }}
                                    inlineTags={true}
                                    inputFieldPosition="bottom"
                                    delimiter={Delimiter.Enter}
                                    delimiterList={[",", "，", ";", " ", "Enter"]}
                                    addOnPaste={true}
                                    addTagsOnBlur={true}
                                />
                            </FormControl>
                            <FormDescription>
                                输入邮箱后按回车添加，支持粘贴多个（逗号分隔）
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                {/* 发送模板 + 回复邮箱 - 第二行 */}
                <div className={cn("grid gap-6 md:grid-cols-2")}>
                    <FormField
                        control={form.control}
                        name="tpl_key"
                        render={({ field }) => (
                            <FormItem>
                                <FormLabel>发送模板 *</FormLabel>
                                <FormControl>
                                    <AutocompleteInput
                                        value={tplSearchKeyword || field.value}
                                        onChange={(value) => {
                                            setTplSearchKeyword(value)
                                            // 如果选中了一个选项，更新表单值
                                            const selectedTpl = tplOptions.find(opt => opt.value === value)
                                            if (selectedTpl) {
                                                field.onChange(value)
                                                setTplSearchKeyword(value)
                                            } else {
                                                // 用户正在输入搜索
                                                field.onChange(value)
                                            }
                                        }}
                                        placeholder="搜索或选择发送模板"
                                        loading={isLoadingTplConfig}
                                        options={tplOptions}
                                        filterOnInput={true}
                                    />
                                </FormControl>
                                <FormDescription>
                                    选择已配置的邮件发送模板
                                </FormDescription>
                                <FormMessage />
                            </FormItem>
                        )}
                    />

                    <FormField
                        control={form.control}
                        name="reply"
                        render={({ field }) => (
                            <FormItem>
                                <FormLabel>回复邮箱（可选）</FormLabel>
                                <FormControl>
                                    <Input
                                        placeholder="输入回复邮箱地址"
                                        {...field}
                                    />
                                </FormControl>
                                <FormDescription>
                                    收件人回复时的邮箱地址，留空则使用默认配置
                                </FormDescription>
                                <FormMessage />
                            </FormItem>
                        )}
                    />
                </div>

                {/* 发送时间 + 最大重试次数 - 同一行 */}
                <div className={cn("grid gap-6 md:grid-cols-2")}>
                    <FormField
                        control={form.control}
                        name="send_time"
                        render={({ field }) => (
                            <FormItem>
                                <FormLabel>发送时间（可选）</FormLabel>
                                <FormControl>
                                    <DateTimePicker
                                        value={field.value}
                                        onChange={field.onChange}
                                        placeholder="选择发送时间（留空则立即发送）"
                                        minDateTime={minDateTime}
                                    />
                                </FormControl>
                                <FormDescription>
                                    设置定时发送时间，留空则立即发送
                                </FormDescription>
                                <FormMessage />
                            </FormItem>
                        )}
                    />

                    <FormField
                        control={form.control}
                        name="max_try"
                        render={({ field }) => (
                            <FormItem>
                                <FormLabel>最大重试次数</FormLabel>
                                <FormControl>
                                    <NumberInput
                                        value={field.value ?? 0}
                                        onChange={field.onChange}
                                        min={0}
                                        max={10}
                                        step={1}
                                    />
                                </FormControl>
                                <FormDescription>
                                    {field.value === 0 ? "不重试" : `发送失败后最多重试 ${field.value} 次`}
                                </FormDescription>
                                <FormMessage />
                            </FormItem>
                        )}
                    />
                </div>

                {/* 模板数据 */}
                <FormField
                    control={form.control}
                    name="data"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>模板数据（JSON）</FormLabel>
                            <FormControl>
                                <div className={cn("border rounded-md overflow-hidden bg-muted/30")}>
                                    <CodeEditor
                                        value={field.value}
                                        language="json"
                                        placeholder='输入模板变量数据，例如: {"name": "张三", "code": "123456"}'
                                        onChange={(evn) => field.onChange(evn.target.value)}
                                        padding={15}
                                        minHeight={150}
                                        style={{
                                            fontSize: 13,
                                            backgroundColor: "hsl(var(--muted) / 0.3)",
                                            fontFamily:
                                                "ui-monospace,SFMono-Regular,SF Mono,Consolas,Liberation Mono,Menlo,monospace",
                                            lineHeight: "1.5",
                                        }}
                                        className={cn("w-full")}
                                    />
                                </div>
                            </FormControl>
                            <FormDescription>
                                输入 JSON 格式的模板变量数据，用于替换模板中的变量
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                {/* 提交按钮 */}
                <div className={cn("flex justify-end gap-2 pt-4")}>
                    <Button
                        type="button"
                        variant="outline"
                        onClick={() => {
                            form.reset()
                            setEmailTags([])
                            setTplSearchKeyword("")
                        }}
                        disabled={sendMutation.isPending}
                    >
                        重置
                    </Button>
                    <Button type="submit" disabled={sendMutation.isPending}>
                        {sendMutation.isPending && (
                            <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />
                        )}
                        发送邮件
                    </Button>
                </div>
            </form>
        </Form>
    )
}
