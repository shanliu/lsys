import {
  adminRuntimeSettingGet,
  adminRuntimeSettingUpdate,
  type AdminRuntimeSettingType,
} from "@shared/apis/admin/file";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
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
import { useToast } from "@shared/contexts/toast-context";
import { cn, formatServerError } from "@shared/lib/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Loader2, Save } from "lucide-react";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import {
  RuntimeSettingFormSchema,
  type RuntimeSettingFormType,
} from "./runtime-setting-schema";

export function AdminRuntimeSettingPage() {
  const queryClient = useQueryClient();
  const { success: showSuccess, error: showError } = useToast();

  // 获取当前配置
  const {
    data: settingData,
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery({
    queryKey: ["adminRuntimeSetting"],
    queryFn: () => adminRuntimeSettingGet(),
  });

  const currentSetting: AdminRuntimeSettingType | undefined =
    settingData?.response;

  const form = useForm<RuntimeSettingFormType>({
    resolver: zodResolver(RuntimeSettingFormSchema),
    defaultValues: {
      local_public_url_prefix: "/files/",
      max_download_concurrency: 10,
      download_timeout_secs: 60,
    },
  });

  // 当数据加载完成时更新表单
  useEffect(() => {
    if (currentSetting) {
      form.reset({
        local_public_url_prefix: currentSetting.local_public_url_prefix,
        max_download_concurrency: currentSetting.max_download_concurrency,
        download_timeout_secs: currentSetting.download_timeout_secs,
      });
    }
  }, [currentSetting, form]);

  const updateMutation = useMutation({
    mutationFn: (param: RuntimeSettingFormType) =>
      adminRuntimeSettingUpdate(param),
    onSuccess: () => {
      showSuccess("运行时配置已更新");
      queryClient.invalidateQueries({ queryKey: ["adminRuntimeSetting"] });
    },
    onError: (err: any) => showError(formatServerError(err, "更新失败")),
  });

  const onSubmit = (data: RuntimeSettingFormType) => {
    updateMutation.mutate(data);
  };

  if (isError) {
    return (
      <div
        className={cn(
          "container mx-auto p-4 lg:px-6 py-5 max-w-[1200px] flex items-center justify-center min-h-[400px]",
        )}
      >
        <CenteredError
          error={error}
          variant="content"
          onReset={() => refetch()}
        />
      </div>
    );
  }

  return (
    <div
      className={cn(
        "container mx-auto p-4 lg:px-6 py-5 max-w-[1200px] space-y-6",
      )}
    >
      {/* 页面标题 */}
      <div className={cn("space-y-1")}>
        <h2 className={cn("text-2xl font-semibold tracking-tight")}>
          文件运行时配置
        </h2>
        <p className={cn("text-sm text-muted-foreground")}>
          管理文件服务的运行时参数，修改后立即生效，无需重启服务
        </p>
      </div>

      {/* 配置表单 */}
      <div
        className={cn(
          "rounded-lg border bg-card text-card-foreground shadow-sm",
        )}
      >
        <div className={cn("p-6")}>
          {isLoading ? (
            <div className={cn("flex items-center justify-center py-12")}>
              <Loader2 className={cn("h-8 w-8 animate-spin text-muted-foreground")} />
            </div>
          ) : (
            <Form {...form}>
              <form
                onSubmit={form.handleSubmit(onSubmit)}
                className={cn("space-y-6")}
              >
                <FormField
                  control={form.control}
                  name="local_public_url_prefix"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>本地公开文件 URL 前缀</FormLabel>
                      <FormControl>
                        <Input
                          placeholder="/file/ 或 https://cdn.example.com/files/"
                          {...field}
                        />
                      </FormControl>
                      <FormDescription>
                        用于生成本地公开文件的访问 URL。支持配置 CDN
                        地址，建议以 "/" 结尾
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="max_download_concurrency"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>最大下载并发数</FormLabel>
                      <FormControl>
                        <Input
                          type="number"
                          min={1}
                          max={100}
                          {...field}
                          value={
                            field.value !== undefined && field.value !== null
                              ? String(field.value)
                              : ""
                          }
                          onChange={(e) =>
                            field.onChange(e.target.valueAsNumber || 1)
                          }
                        />
                      </FormControl>
                      <FormDescription>
                        控制同时进行的文件下载任务数量，可根据服务器性能和网络带宽调整
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="download_timeout_secs"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>下载超时时间（秒）</FormLabel>
                      <FormControl>
                        <Input
                          type="number"
                          min={10}
                          max={3600}
                          {...field}
                          value={
                            field.value !== undefined && field.value !== null
                              ? String(field.value)
                              : ""
                          }
                          onChange={(e) =>
                            field.onChange(e.target.valueAsNumber || 60)
                          }
                        />
                      </FormControl>
                      <FormDescription>
                        单个文件下载的最大允许时间，可根据网络环境和文件大小调整
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <div className={cn("flex justify-end pt-4")}>
                  <Button
                    type="submit"
                    disabled={updateMutation.isPending || isLoading}
                  >
                    {updateMutation.isPending && (
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    )}
                    <Save className={cn("mr-2 h-4 w-4")} />
                    保存配置
                  </Button>
                </div>
              </form>
            </Form>
          )}
        </div>
      </div>

      {/* 配置说明 */}
      <div
        className={cn(
          "rounded-lg border bg-muted/50 p-4 text-sm text-muted-foreground",
        )}
      >
        <h3 className={cn("font-medium text-foreground mb-2")}>配置说明</h3>
        <ul className={cn("space-y-1 list-disc list-inside")}>
          <li>
            <strong>URL 前缀</strong>：用于生成本地公开文件的访问链接，支持配置
            CDN 加速
          </li>
          <li>
            <strong>并发数</strong>：控制同时下载的文件数量，避免资源耗尽
          </li>
          <li>
            <strong>超时时间</strong>：防止下载任务长时间占用资源
          </li>
          <li>所有配置修改后立即生效，无需重启服务</li>
        </ul>
      </div>
    </div>
  );
}
