import { getSiteConfig, setSiteConfig, SiteConfigSetParamSchema, SiteConfigSetParamType, SiteConfigGetResType } from '@shared/apis/admin/config'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { TimeoutInput } from '@shared/components/custom/input/timeout-input'
import { Button } from '@shared/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@shared/components/ui/card'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@shared/components/ui/form'
import { RadioGroup, RadioGroupItem } from '@shared/components/ui/radio-group'
import { Textarea } from '@shared/components/ui/textarea'
import { useToast } from '@shared/contexts/toast-context'
import { ConfigNavContainer } from '@apps/main/features/admin/components/ui/config-nav'
import { cn, formatServerError } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Save, Settings } from 'lucide-react'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { configModuleConfig } from '../nav-info'

const DEFAULT_CONFIG: SiteConfigGetResType = {
    config: {
        dis_old_password: 'false',
        site_tips: '',
        timeout: 0,
    }
}

export function SiteConfigPage() {
    const toast = useToast()
    const queryClient = useQueryClient()

    const {
        data: configData,
        isLoading,
        isError,
        error,
    } = useQuery({
        queryKey: ['site-config'],
        queryFn: async ({ signal }) => {
            const result = await getSiteConfig({ signal })
            return result
        },
    })

    const config = configData?.response || DEFAULT_CONFIG

    const form = useForm<SiteConfigSetParamType>({
        resolver: zodResolver(SiteConfigSetParamSchema),
        defaultValues: {
            site_tips: '',
            password_timeout: 0,
            disable_old_password: false,
        },
    })

    // 当数据加载完成后，更新表单默认值
    useEffect(() => {
        if (config?.config) {
            form.reset({
                site_tips: config.config.site_tips,
                password_timeout: config.config.timeout,
                disable_old_password: config.config.dis_old_password === '1',
            })
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [config])

    const saveMutation = useMutation({
        mutationFn: (data: SiteConfigSetParamType) => setSiteConfig(data),
        onSuccess: () => {
            toast.success('站点配置保存成功')
            queryClient.invalidateQueries({ queryKey: ['site-config'] })
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        },
    })

    const onSubmit = (data: SiteConfigSetParamType) => {
        saveMutation.mutate(data)
    }

    const refreshData = () => {
        queryClient.refetchQueries({ queryKey: ['site-config'] })
    }

    if (isLoading) {
        return (
            <ConfigNavContainer {...configModuleConfig}>
                <CenteredLoading variant="content"/>
            </ConfigNavContainer>
        )
    }

    if (isError) {
        return (
            <ConfigNavContainer {...configModuleConfig}>
                <CenteredError variant="content" error={error} onReset={refreshData} />
            </ConfigNavContainer>
        )
    }

    return (
        <ConfigNavContainer {...configModuleConfig}>
            <div className="space-y-6">
                <Form {...form}>
                    <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
                        <Card>
                            <CardHeader>
                                <CardTitle className={cn('flex items-center gap-2')}>
                                    <Settings className={cn('h-5 w-5')} />
                                    站点配置
                                </CardTitle>
                                <CardDescription>配置站点提示信息和密码策略</CardDescription>
                            </CardHeader>
                            <CardContent className={cn('space-y-4')}>
                                <FormField
                                    control={form.control}
                                    name="site_tips"
                                    render={({ field }) => (
                                        <FormItem>
                                            <FormLabel>站点提示信息</FormLabel>
                                            <FormControl>
                                                <Textarea placeholder="请输入站点提示信息" {...field} />
                                            </FormControl>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />
                                <FormField
                                    control={form.control}
                                    name="password_timeout"
                                    render={({ field }) => (
                                        <FormItem>
                                            <FormLabel>密码超时时间（秒）</FormLabel>
                                            <FormControl>
                                                <TimeoutInput
                                                    value={field.value}
                                                    onChange={field.onChange}
                                                    placeholder="请输入密码超时时间"
                                                />
                                            </FormControl>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />
                                <FormField
                                    control={form.control}
                                    name="disable_old_password"
                                    render={({ field }) => (
                                        <FormItem className="pt-2">
                                            <div className="flex items-center space-x-4">
                                                <FormLabel>禁用旧密码</FormLabel>
                                                <FormControl>
                                                    <RadioGroup
                                                        onValueChange={(value) => field.onChange(value === 'true')}
                                                        value={field.value ? 'true' : 'false'}
                                                        className="flex items-center space-x-4"
                                                    >
                                                        <div className="flex items-center space-x-2">
                                                            <RadioGroupItem value="false" id="disable-no" />
                                                            <label htmlFor="disable-no" className="cursor-pointer">否</label>
                                                        </div>
                                                        <div className="flex items-center space-x-2">
                                                            <RadioGroupItem value="true" id="disable-yes" />
                                                            <label htmlFor="disable-yes" className="cursor-pointer">是</label>
                                                        </div>
                                                    </RadioGroup>
                                                </FormControl>
                                            </div>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />
                                <div className="pt-4">
                                    <Button type="submit" variant="outline" disabled={saveMutation.isPending}>
                                        <Save className={cn('mr-2 h-4 w-4')} />
                                        {saveMutation.isPending ? '保存中...' : '保存配置'}
                                    </Button>
                                </div>
                            </CardContent>
                        </Card>
                    </form>
                </Form>
            </div>
        </ConfigNavContainer>
    )
}
