import { LimitDataParam } from '@shared/types/base-schema';
import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const UserChangeLogFilterBaseSchema = z.object({
  log_type: z.string().optional(),
  add_user_id: NumberParamSchema,
});

// URL 参数 schema，包含分页参数
export const UserChangeLogFilterParamSchema = UserChangeLogFilterBaseSchema.extend(LimitDataParam);

// 表单 schema，使用 coerce 来自动处理字符串到数字的转换
export const UserChangeLogFilterFormSchema = z.object({
  log_type: z.string().optional(),
  add_user_id: NumberParamSchema.catch(undefined),
});

export type UserChangeLogFilterParamType = z.infer<typeof UserChangeLogFilterParamSchema>;
