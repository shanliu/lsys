import { z } from 'zod';

// 设置用户信息的表单数据结构
export const SetInfoSchema = z.object({
    nikename: z.string().trim().max(50, '昵称最多50个字符').optional(),
    gender: z.coerce.number().min(0).max(2).optional(), // 0: 未知, 1: 男, 2: 女
    headimg: z.string().trim().url('请输入有效的头像URL').optional().or(z.literal('')),
    birthday: z.string().optional(), // YYYY-MM-DD 格式
});

export type SetInfoType = z.infer<typeof SetInfoSchema>;
