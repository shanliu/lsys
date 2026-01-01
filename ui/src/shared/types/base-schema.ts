import { z } from "zod";

export const BoolSchema = z.preprocess((val) => {
  if (
    val === "1" ||
    val === "true" ||
    val === "True" ||
    val === "yes" ||
    val === "on" ||
    val === 1 ||
    val === true
  )
    return true;
  return false;
}, z.boolean()) as z.ZodType<boolean>;
export type BoolType = z.infer<typeof BoolSchema>;



export const UnixTimestampSchema = z.preprocess((val) => {
  if (val === "0" || val === 0 || val == null) return null; // 返回 null 以便 nullable 处理
  if (val instanceof Date) return val;
  if (typeof val === "number") {
    return new Date(val * 1000);
  }
  if (typeof val === "string") {
    if (/^\d+$/.test(val)) {
      const num = Number(val);
      return new Date(num * 1000);
    }
    const d = new Date(val); // 尝试解析 ISO 字符串
    if (!isNaN(d.getTime())) return d;
  }
  // 如果都失败了，返回 null 表示无效时间戳
  console.warn("Invalid timestamp value:", val);
  return null;
}, z.date().nullable()) as z.ZodType<Date | null>;
export type UnixTimestampType = z.infer<typeof UnixTimestampSchema>;

export const CaptChaSchema = z.object({
  code: z.string().min(1, "请输入验证码."),
  key: z.string().min(1, "验证码标识丢失,请刷新页面重试."),
});

export type CaptchaType = z.infer<typeof CaptChaSchema>;

export const CaptChaParam = {
  captcha: CaptChaSchema.optional(),
};

// 分页字段定义（用于 FilterParamSchema.extend() 和 PageParam）
export const PageDataParam = {
  page: z.coerce.number().min(1, "页码最小为1").optional(),
  limit: z.coerce
    .number()
    .min(1, "返回结果最小为1")
    .max(100, "返回值最大为100")
    .optional(),
} as const;

export const PageParam = {
  page: z.object(PageDataParam),
  count_num: BoolSchema.default(false),
};

export type PageDataType = z.infer<typeof PageParam.page>;

// 偏移分页字段定义（用于 FilterParamSchema.extend() 和 LimitParam）
export const LimitDataParam = {
  eq_pos: z.coerce.boolean().optional(), // 取值时是否包含pos位置的数据
  pos: z.coerce.number().min(0, "起始位置最小为0").nullable().optional(), //可选。第一页传null。
  limit: z.coerce
    .number()
    .min(1, "返回结果最小为1")
    .max(100, "返回值最大为100")
    .optional(),
  forward: BoolSchema.optional(), //数据正序还是倒序
  more: BoolSchema.optional(), //尝试检查是否还有下一页数据
} as const;

export const LimitParam = {
  limit: z.object(LimitDataParam),
  count_num: BoolSchema.default(false),
};

export type LimitDataType = z.infer<typeof LimitParam.limit>;

export const PageRes = {
  total: z.coerce.number().nullable().optional(),
};

export const LimitRes = {
  next: z.coerce.number().nullable().optional(),
  total: z.coerce.number().nullable().optional(),
};

// 用户数据结构
export const UserDataRes = z.object({
  /** 用户所属应用ID */
  app_id: z.coerce.number(),
  /** 用户ID */
  id: z.coerce.number(),
  /** 用户账号 */
  user_account: z.string(),
  /** 用户数据 */
  user_data: z.string(),
  /** 用户昵称 */
  user_nickname: z.string(),
});

export type UserDataType = z.infer<typeof UserDataRes>;




// 其余可解析的数字/数字字符串 -> number
export const NumberParamSchema = z.preprocess((val) => {
    // 空字符串应视为未提供 -> undefined（用于移除 URL 参数）
    if (val === '' || (typeof val === 'string' && val.trim() === '')) return undefined;
    if (val === 'null' || val === null) return null;
    if (val === 'undefined' || val === undefined) return undefined;
    if (val instanceof Date) return Number.isNaN(val.getTime()) ? null : val.getTime();
    if (typeof val === 'number') return Number.isNaN(val) ? null : val;
    if (typeof val === 'string') {
        const trimmed = val.trim();
        const num = Number(trimmed);
        return Number.isNaN(num) ? null : num;
    }
    // 其它类型视为无效 -> null
    return null;
}, z.number().nullable().optional()) as z.ZodType<number | null | undefined>;
export type NumberParamType = z.infer<typeof NumberParamSchema>;

// 通用布尔参数 Schema 处理：
// '' 或空白 => undefined（移除 URL 参数）
// 'undefined' => undefined，'null' 或 null => null
// '1'/'true'/'True'/'yes'/'on' => true
// '0'/'false'/'False'/'no'/'off' => false
// 其它类型按 JS 语义：boolean、number 0/1、字符串解析
export const BoolParamSchema = z.preprocess((val) => {
    if (val === undefined || val === 'undefined') return undefined;
    if (val === null || val === 'null') return null;
    if (typeof val === 'string') {
        const s = val.trim();
        if (s === '') return undefined;
        if (s === '1' || s.toLowerCase() === 'true' || s.toLowerCase() === 'yes' || s.toLowerCase() === 'on') return true;
        if (s === '0' || s.toLowerCase() === 'false' || s.toLowerCase() === 'no' || s.toLowerCase() === 'off') return false;
        // 其它字符串尝试数字化
        const num = Number(s);
        if (!Number.isNaN(num)) return num !== 0;
        return undefined; // 无法解析的字符串视为未提供
    }
    if (typeof val === 'number') return val !== 0;
    if (typeof val === 'boolean') return val;
    return undefined;
}, z.boolean().optional().nullable()) as z.ZodType<boolean | null | undefined>;
export type BoolParamType = z.infer<typeof BoolParamSchema>;

export const MobileSchema = z
    .string()
    .regex(/^1[3-9]\d{9}$/, "请输入正确的手机号码");
export type MobileType = z.infer<typeof MobileSchema>;

// 邮箱验证
export const EmailSchema = z
    .string()
    .regex(/^[^\s@]+@[^\s@]+\.[^\s@]+$/, "请输入有效的邮箱地址");
export type EmailType = z.infer<typeof EmailSchema>;
