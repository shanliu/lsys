import { getWechatOAuthConfig, setWechatOAuthConfig, WechatOAuthConfigSetParamSchema, WechatOAuthConfigSetParamType, WechatOAuthConfigGetResType } from '@shared/apis/admin/config'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { Button } from '@shared/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@shared/components/ui/card'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@shared/components/ui/form'
import { Input } from '@shared/components/ui/input'
import { useToast } from '@shared/contexts/toast-context'
import { ConfigNavContainer } from '@apps/main/features/admin/components/ui/config-nav'
import { cn, formatServerError } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Lock, Save } from 'lucide-react'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { configModuleConfig } from '../nav-info'

const DEFAULT_CONFIG: WechatOAuthConfigGetResType = {
  config: {
    app_id: '',
    app_secret: '',
  }
}

export function OAuthConfigPage() {
  const toast = useToast()
  const queryClient = useQueryClient()

  const {
    data: configData,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: ['wechat-oauth-config'],
    queryFn: async ({ signal }) => {
      const result = await getWechatOAuthConfig({ signal })
      return result
    },
  })

  const config = configData?.response || DEFAULT_CONFIG

  const form = useForm<WechatOAuthConfigSetParamType>({
    resolver: zodResolver(WechatOAuthConfigSetParamSchema),
    defaultValues: {
      app_id: '',
      app_secret: '',
    },
  })

  // 当数据加载完成后，更新表单默认值
  useEffect(() => {
    if (config?.config) {
      form.reset({
        app_id: config.config.app_id,
        app_secret: config.config.app_secret,
      })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [config])

  const saveMutation = useMutation({
    mutationFn: (data: WechatOAuthConfigSetParamType) => setWechatOAuthConfig(data),
    onSuccess: () => {
      toast.success('微信OAuth配置保存成功')
      queryClient.invalidateQueries({ queryKey: ['wechat-oauth-config'] })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  const onSubmit = (data: WechatOAuthConfigSetParamType) => {
    saveMutation.mutate(data)
  }

  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ['wechat-oauth-config'] })
  }

  if (isLoading) {
    return (
      <ConfigNavContainer {...configModuleConfig}>
        <CenteredLoading variant="content" />
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
                  <Lock className={cn('h-5 w-5')} />
                  微信OAuth配置
                </CardTitle>
               </CardHeader>
              <CardContent className={cn('space-y-4')}>
                <FormField
                  control={form.control}
                  name="app_id"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>微信应用ID</FormLabel>
                      <FormControl>
                        <Input
                          placeholder="请输入微信应用ID"
                          {...field}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="app_secret"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>微信应用密钥</FormLabel>
                      <FormControl>
                        <Input
                          placeholder="请输入微信应用密钥"
                          {...field}
                        />
                      </FormControl>
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
