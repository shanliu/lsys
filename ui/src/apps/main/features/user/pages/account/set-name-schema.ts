import { z } from 'zod';

// 设置用户名的表单数据结构
export const SetNameSchema = z.object({
    username: z.string().min(3, '用户名至少3个字符').max(20, '用户名最多20  个字符'),
});

export type SetNameForm = z.infer<typeof SetNameSchema>;
