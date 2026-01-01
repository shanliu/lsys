import { LimitDataParam } from '@shared/types/base-schema';
import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema，app_id 默认为 0（系统）
const LoginLogFilterBaseSchema = z.object({
  app_id: NumberParamSchema.default(0),
  oauth_app_id: NumberParamSchema,
  user_id: NumberParamSchema,
  is_enable: z.boolean().optional().nullable(),
});

// URL 参数 schema，包含分页参数，app_id 默认为 0（系统）
export const LoginLogFilterParamSchema = LoginLogFilterBaseSchema.extend({
  ...LimitDataParam,
  app_id: NumberParamSchema.catch(0), // 默认为 0 表示系统
});

// 表单 schema，使用 coerce 来自动处理字符串到数字的转换，app_id 默认为 0
export const LoginLogFilterFormSchema = z.object({
  app_id: NumberParamSchema.catch(0),
  oauth_app_id: NumberParamSchema.catch(undefined),
  user_id: NumberParamSchema.catch(undefined),
  is_enable: z.coerce.boolean().optional().catch(undefined),
});

export type LoginLogFilterParamType = z.infer<typeof LoginLogFilterParamSchema>;

// 偏移分页状态类型
export type LoginOffsetPaginationType = {
  pos: number | null
  limit: number
  forward: boolean
  more: boolean
  eq_pos: boolean
};
