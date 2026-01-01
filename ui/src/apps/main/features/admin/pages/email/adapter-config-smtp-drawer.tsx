import {
  systemSenderMailerSmtpConfigAdd,
  systemSenderMailerSmtpConfigCheck,
  systemSenderMailerSmtpConfigEdit,
  type SystemSenderMailerSmtpConfigItemType,
} from "@shared/apis/admin/sender-mailer";
import { PasswordInput } from "@shared/components/custom/input/password-input";
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
  DrawerTitle,
} from "@apps/main/components/local/drawer";
import { useToast } from "@shared/contexts/toast-context";
import { cn, formatServerError } from "@shared/lib/utils";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import React from "react";
import { useForm } from "react-hook-form";
import {
  SmtpConfigFormSchema,
  type SmtpConfigFormType,
} from "./adapter-config-smtp-schema";

interface SmtpConfigDrawerProps {
  config?: SystemSenderMailerSmtpConfigItemType;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function SmtpConfigDrawer({ config, open, onOpenChange }: SmtpConfigDrawerProps) {
  const toast = useToast();
  const queryClient = useQueryClient();
  const [isCheckingConnection, setIsCheckingConnection] = React.useState(false);
  const isEdit = !!config;

  const form = useForm<SmtpConfigFormType>({
    resolver: zodResolver(SmtpConfigFormSchema),
    defaultValues: {
      name: "",
      host: "",
      port: 465,
      timeout: 30,
      email: "",
      user: "",
      password: "",
      tls_domain: "",
      branch_limit: 1,
    },
  });

  // 添加配置
  const addMutation = useMutation({
    mutationFn: (data: SmtpConfigFormType) => systemSenderMailerSmtpConfigAdd(data),
    onSuccess: () => {
      toast.success("SMTP配置添加成功");
      queryClient.invalidateQueries({ queryKey: ["smtp-config-list"] });
      onOpenChange(false);
      form.reset();
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    },
  });

  // 编辑配置
  const editMutation = useMutation({
    mutationFn: (data: SmtpConfigFormType & { id: number }) =>
      systemSenderMailerSmtpConfigEdit(data),
    onSuccess: () => {
      toast.success("SMTP配置更新成功");
      queryClient.invalidateQueries({ queryKey: ["smtp-config-list"] });
      onOpenChange(false);
      form.reset();
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    },
  });

  // 检查SMTP连接
  const handleCheckConnection = async () => {
    const values = form.getValues();

    // 验证必填字段
    const errors: string[] = [];

    if (!values.host) errors.push("SMTP服务器地址");
    if (!values.port) errors.push("端口");
    if (!values.timeout) errors.push("超时时间");
    if (!values.email) errors.push("发件人邮箱");
    if (!values.user) errors.push("用户名");
    if (!values.password) errors.push("密码");

    if (errors.length > 0) {
      toast.error(`请先填写以下字段: ${errors.join("、")}`);
      return;
    }

    setIsCheckingConnection(true);
    try {
      const result = await systemSenderMailerSmtpConfigCheck({
        host: values.host,
        port: values.port,
        timeout: values.timeout,
        email: values.email,
        user: values.user,
        password: values.password,
        tls_domain: values.tls_domain,
      });

      if (result.status) {
        toast.success("SMTP连接检查成功");
      } else {
        toast.error("SMTP连接检查失败");
      }
    } catch (error: any) {
      toast.error(formatServerError(error));
    } finally {
      setIsCheckingConnection(false);
    }
  };

  // 提交表单
  const onSubmit = (data: SmtpConfigFormType) => {
    if (config) {
      editMutation.mutate({ ...data, id: config.id });
    } else {
      addMutation.mutate(data);
    }
  };

  const isSubmitting = addMutation.isPending || editMutation.isPending;

  // Reset form when config changes or dialog opens
  React.useEffect(() => {
    if (open && config) {
      form.reset({
        name: config.name,
        host: config.host,
        port: config.port,
        timeout: config.timeout,
        email: config.email,
        user: config.user,
        password: config.password || "",
        tls_domain: config.tls_domain || "",
        branch_limit: 1, // API返回中没有这个字段，使用默认值
      });
    } else if (open && !config) {
      form.reset({
        name: "",
        host: "",
        port: 465,
        timeout: 30,
        email: "",
        user: "",
        password: "",
        tls_domain: "",
        branch_limit: 1,
      });
    }
  }, [open, config, form]);

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>{isEdit ? "编辑SMTP配置" : "新增SMTP配置"}</DrawerTitle>
          <DrawerDescription>
            {isEdit ? "修改现有的SMTP服务器配置" : "添加新的SMTP服务器配置"}
          </DrawerDescription>
        </DrawerHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 mt-6">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>配置名称</FormLabel>
                  <FormControl>
                    <Input placeholder="例如：企业邮箱" {...field} />
                  </FormControl>
                  <FormDescription>用于标识此SMTP配置的名称</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="host"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>SMTP服务器地址</FormLabel>
                  <FormControl>
                    <Input placeholder="例如：smtp.qq.com" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="port"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>端口</FormLabel>
                    <FormControl>
                      <Input
                        type="number"
                        placeholder="465"
                        min="1"
                        max="65535"
                        {...field}
                        onChange={(e) => field.onChange(e.target.valueAsNumber)}
                      />
                    </FormControl>
                    <FormDescription>范围: 1-65535</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="timeout"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>超时时间(秒)</FormLabel>
                    <FormControl>
                      <Input
                        type="number"
                        placeholder="30"
                        min="1"
                        max="60"
                        {...field}
                        onChange={(e) => field.onChange(e.target.valueAsNumber)}
                      />
                    </FormControl>
                    <FormDescription>范围: 1-60秒</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <FormField
              control={form.control}
              name="email"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>发件人邮箱</FormLabel>
                  <FormControl>
                    <Input type="email" placeholder="example@qq.com" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="user"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>SMTP用户名</FormLabel>
                  <FormControl>
                    <Input placeholder="通常与邮箱地址相同" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="password"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>SMTP密码</FormLabel>
                  <FormControl>
                    <PasswordInput placeholder="输入SMTP密码或授权码" {...field} />
                  </FormControl>
                  <FormDescription>通常需要使用授权码，而非邮箱登录密码</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="tls_domain"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>TLS域名</FormLabel>
                  <FormControl>
                    <Input placeholder="可选，留空使用默认" {...field} />
                  </FormControl>
                  <FormDescription>TLS加密域名，一般可留空</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="branch_limit"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>单次发送量</FormLabel>
                  <FormControl>
                    <Input
                      type="number"
                      placeholder="1"
                      {...field}
                      onChange={(e) => field.onChange(e.target.valueAsNumber)}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className="flex gap-3 pt-4">
              <Button
                type="button"
                variant="outline"
                className={cn("flex-1")}
                onClick={handleCheckConnection}
                disabled={isCheckingConnection || isSubmitting}
              >
                {isCheckingConnection && (
                  <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />
                )}
                检查连接
              </Button>
              <Button type="submit" className={cn("flex-1")} disabled={isSubmitting}>
                {isSubmitting && <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />}
                {isEdit ? "更新配置" : "添加配置"}
              </Button>
            </div>
          </form>
        </Form>
      </DrawerContent>
    </Drawer>
  );
}
