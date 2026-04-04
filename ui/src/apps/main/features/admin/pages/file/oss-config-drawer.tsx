import {
  adminOssConfigAdd,
  adminOssConfigEdit,
  type AdminOssConfigItemType,
} from "@shared/apis/admin/file";
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from "@apps/main/components/local/drawer";
import { PasswordInput } from "@shared/components/custom/input/password-input";
import { Button } from "@shared/components/ui/button";
import {
  Form,
  FormControl,
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
import { useToast } from "@shared/contexts/toast-context";
import { cn, formatServerError } from "@shared/lib/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import React from "react";
import { useForm } from "react-hook-form";
import {
  OssConfigFormSchema,
  type OssConfigFormType,
} from "./oss-config-schema";

interface OssConfigDrawerProps {
  config?: AdminOssConfigItemType | null;
  providerTypes: Array<{ key: string; val: string }>;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}

export function OssConfigDrawer({
  config,
  providerTypes,
  open,
  onOpenChange,
  onSuccess,
}: OssConfigDrawerProps) {
  const toast = useToast();
  const queryClient = useQueryClient();
  const isEdit = !!config;

  const form = useForm<OssConfigFormType>({
    resolver: zodResolver(OssConfigFormSchema),
    defaultValues: {
      name: "",
      config_key: "",
      provider_type: "",
      endpoint: "",
      bucket: "",
      access_key: "",
      secret_key: "",
      region: "",
      timeout_secs: 30,
    },
  });

  // 当选项改变或打开时重置表单
  React.useEffect(() => {
    if (open) {
      if (config) {
        const providerConfig = (config.provider_config || {}) as Record<
          string,
          any
        >;
        form.reset({
          name: config.name,
          config_key: config.config_key,
          provider_type: config.provider_type,
          endpoint: providerConfig.endpoint || "",
          bucket: providerConfig.bucket || "",
          access_key: providerConfig.access_key || "",
          secret_key: providerConfig.secret_key || "",
          region: providerConfig.region || "",
          timeout_secs:
            providerConfig.timeout_secs !== undefined
              ? Number(providerConfig.timeout_secs)
              : 30,
        });
      } else {
        form.reset({
          name: "",
          config_key: "",
          provider_type: providerTypes[0]?.key || "",
          endpoint: "",
          bucket: "",
          access_key: "",
          secret_key: "",
          region: "",
          timeout_secs: 30,
        });
      }
    }
  }, [open, config, form, providerTypes]);

  const addMutation = useMutation({
    mutationFn: (param: Parameters<typeof adminOssConfigAdd>[0]) =>
      adminOssConfigAdd(param),
    onSuccess: () => {
      toast.success("OSS 配置新增成功");
      queryClient.invalidateQueries({ queryKey: ["adminOssConfigList"] });
      onSuccess();
    },
    onError: (error: any) => toast.error(formatServerError(error, "新增失败")),
  });

  const editMutation = useMutation({
    mutationFn: (param: Parameters<typeof adminOssConfigEdit>[0]) =>
      adminOssConfigEdit(param),
    onSuccess: () => {
      toast.success("OSS 配置更新成功");
      queryClient.invalidateQueries({ queryKey: ["adminOssConfigList"] });
      onSuccess();
    },
    onError: (error: any) => toast.error(formatServerError(error, "更新失败")),
  });

  const onSubmit = (data: OssConfigFormType) => {
    const providerConfig: Record<string, any> = {
      endpoint: data.endpoint,
      bucket: data.bucket,
      access_key: data.access_key,
      secret_key: data.secret_key,
      timeout_secs: data.timeout_secs,
    };

    if (data.provider_type === "aws-s3") {
      providerConfig.region = data.region;
    }

    if (isEdit && config) {
      editMutation.mutate({
        id: config.id,
        name: data.name,
        provider_config: providerConfig,
      });
    } else {
      addMutation.mutate({
        name: data.name,
        config_key: data.config_key,
        provider_type: data.provider_type,
        provider_config: providerConfig,
      });
    }
  };

  const isSubmitting = addMutation.isPending || editMutation.isPending;
  const currentProviderType = form.watch("provider_type");

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>
            {isEdit ? "编辑 OSS 配置" : "新增 OSS 配置"}
          </DrawerTitle>
          <DrawerDescription>
            {isEdit ? "修改现有的 OSS 云存储配置" : "配置新的 OSS 云存储"}
          </DrawerDescription>
        </DrawerHeader>

        <Form {...form}>
          <form
            onSubmit={form.handleSubmit(onSubmit)}
            className="space-y-4 mt-6"
          >
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>配置名称</FormLabel>
                  <FormControl>
                    <Input placeholder="例如：阿里云-生产环境" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="config_key"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    配置标识
                    {isEdit && (
                      <span
                        className={cn("text-xs text-muted-foreground ml-1")}
                      >
                        （不可修改）
                      </span>
                    )}
                  </FormLabel>
                  <FormControl>
                    <Input
                      placeholder="小写字母数字连字符，例如：aliyun-prod"
                      {...field}
                      disabled={isEdit}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="provider_type"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    厂商类型
                    {isEdit && (
                      <span
                        className={cn("text-xs text-muted-foreground ml-1")}
                      >
                        （不可修改）
                      </span>
                    )}
                  </FormLabel>
                  <Select
                    onValueChange={field.onChange}
                    defaultValue={field.value}
                    value={field.value}
                    disabled={isEdit}
                  >
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="选择厂商类型" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {providerTypes.map((t) => (
                        <SelectItem key={t.key} value={t.key}>
                          {t.val || t.key}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className={cn("pt-2 border-t mt-4 pb-2")}>
              <h4 className={cn("text-sm font-medium mb-4")}>厂商认证信息</h4>

              <div className={cn("flex flex-col gap-4")}>
                <FormField
                  control={form.control}
                  name="endpoint"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Endpoint</FormLabel>
                      <FormControl>
                        <Input
                          placeholder="例如：oss-cn-hangzhou.aliyuncs.com"
                          {...field}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                {currentProviderType === "aws-s3" && (
                  <FormField
                    control={form.control}
                    name="region"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Region</FormLabel>
                        <FormControl>
                          <Input placeholder="例如：us-east-1" {...field} />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                )}

                <FormField
                  control={form.control}
                  name="bucket"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Bucket</FormLabel>
                      <FormControl>
                        <Input placeholder="存储桶名称" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="access_key"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Access Key</FormLabel>
                      <FormControl>
                        <Input placeholder="Access Key ID" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="secret_key"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Secret Key</FormLabel>
                      <FormControl>
                        <PasswordInput
                          placeholder="Access Key Secret"
                          {...field}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="timeout_secs"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>超时时间 (秒)</FormLabel>
                      <FormControl>
                        <Input
                          type="number"
                          {...field}
                          value={
                            field.value !== undefined && field.value !== null
                              ? String(field.value)
                              : ""
                          }
                          onChange={(e) =>
                            field.onChange(e.target.valueAsNumber || 0)
                          }
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>
            </div>

            <div className={cn("flex gap-3 pt-6")}>
              <Button
                type="submit"
                className={cn("flex-1")}
                disabled={isSubmitting}
              >
                {isSubmitting && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                {isEdit ? "保存修改" : "立即创建"}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={isSubmitting}
              >
                取消
              </Button>
            </div>
          </form>
        </Form>
      </DrawerContent>
    </Drawer>
  );
}
