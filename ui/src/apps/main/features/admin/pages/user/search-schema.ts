import { LimitDataParam } from '@shared/types/base-schema';
import { z } from 'zod';

// 解析布尔值字符串的 schema
const BoolStringSchema = z
  .union([z.boolean(), z.string()])
  .optional()
  .transform((val) => {
    if (val === undefined || val === '' || val === null) return undefined;
    if (typeof val === 'boolean') return val;
    if (val === 'true') return true;
    if (val === 'false') return false;
    return undefined;
  });

// 基础过滤器字段 schema
const UserAccountFilterBaseSchema = z.object({
  key_word: z.string().optional(),
  enable: BoolStringSchema,
});

// URL 参数 schema，包含分页参数
export const UserAccountFilterParamSchema = UserAccountFilterBaseSchema.extend(LimitDataParam);

// 表单过滤器 schema（不包含分页参数）
export const UserAccountFilterFormSchema = z.object({
  key_word: z.string().optional(),
  enable: BoolStringSchema,
});

export type UserAccountFilterParamType = z.infer<typeof UserAccountFilterParamSchema>;
