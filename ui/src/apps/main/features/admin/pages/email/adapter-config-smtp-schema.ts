import { z } from 'zod';

// 过滤表单 Schema
export const SmtpConfigFilterFormSchema = z.object({
  name: z.string().optional(),
});

export type SmtpConfigFilterParamType = z.infer<typeof SmtpConfigFilterFormSchema>;

// SMTP配置表单 Schema
export const SmtpConfigFormSchema = z.object({
  name: z.string().min(1, "配置名称不能为空"),
  host: z.string().min(1, "SMTP服务器地址不能为空"),
  port: z.coerce.number().min(1, "端口必须在1-65535之间").max(65535, "端口必须在1-65535之间"),
  timeout: z.coerce.number().min(1, "超时时间必须在1-60秒之间").max(60, "超时时间必须在1-60秒之间"),
  email: z.string().email("请输入有效的邮箱地址"),
  user: z.string().min(1, "用户名不能为空"),
  password: z.string().min(1, "密码不能为空"),
  tls_domain: z.string().optional(),
  branch_limit: z.coerce.number().min(0, "分支限制不能小于0"),
});

export type SmtpConfigFormType = z.infer<typeof SmtpConfigFormSchema>;
