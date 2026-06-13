import { z } from 'zod';

export const RuntimeSettingFormSchema = z.object({
  local_public_url_prefix: z
    .string()
    .min(1, 'URL 前缀不能为空')
    .max(255, 'URL 前缀最长 255 字符'),
  max_download_concurrency: z
    .number()
    .int('必须是整数')
    .min(1, '并发数至少为 1')
    .max(100, '并发数最大为 100'),
  download_timeout_secs: z
    .number()
    .int('必须是整数')
    .min(10, '超时时间至少 10 秒')
    .max(3600, '超时时间最大 3600 秒（1 小时）'),
});

export type RuntimeSettingFormType = z.infer<typeof RuntimeSettingFormSchema>;
