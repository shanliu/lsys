import { type ExternalAccountDataType } from '@shared/apis/user/profile';
import { z } from 'zod';

// 外部账号页面搜索参数验证
export const externalSearchSchema = z.object({
  action: z.enum(['bind']).optional(),
});

export type ExternalSearchType = z.infer<typeof externalSearchSchema>;

// 外部账号数据类型
export type ExternalAccountData = ExternalAccountDataType;

// 外部登录类型数据
export interface ExternalLoginType {
  key: string;
  name: string;
  icon?: string;
}
