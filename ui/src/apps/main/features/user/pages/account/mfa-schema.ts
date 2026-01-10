import { z } from 'zod';

// MFA绑定验证码表单数据验证
export const mfaBindFormSchema = z.object({
    code: z.string().trim().min(1, '请输入验证码').min(4, '验证码长度不足'),
});

export type MfaBindFormType = z.infer<typeof mfaBindFormSchema>;

// MFA解绑密码表单数据验证
export const mfaUnbindFormSchema = z.object({
    password: z.string().trim().min(1, '请输入密码'),
});

export type MfaUnbindFormType = z.infer<typeof mfaUnbindFormSchema>;
