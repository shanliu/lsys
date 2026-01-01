import { z } from 'zod';

// 创建应用的表单数据结构
export const AppCreateSchema = z.object({
  parent_app_id: z.coerce.number().nullable().optional(),
  name: z.string().min(1, '应用名称不能为空').max(50, '应用名称不能超过50个字符'),
  client_id: z.string().min(1, '应用标识不能为空').max(50, '应用标识不能超过50个字符'),
});

export type AppCreateForm = z.infer<typeof AppCreateSchema>;
