import { PageDataParam } from '@shared/types/base-schema';
import { z } from "zod";

// 基础过滤器字段 schema
const BarcodeCreateConfigFilterBaseSchema = z.object({
    barcode_type: z.string().optional(),
    status: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const BarcodeCreateConfigFilterParamSchema = BarcodeCreateConfigFilterBaseSchema.extend(PageDataParam);

// 表单 schema
export const BarcodeCreateConfigFilterFormSchema = z.object({
    barcode_type: z.string().optional(),
    status: z.string().optional(),
});

export type BarcodeCreateConfigFilterParamType = z.infer<typeof BarcodeCreateConfigFilterParamSchema>;

// 新增/编辑配置表单 Schema
export const BarcodeCreateConfigFormSchema = z.object({
    barcode_type: z.string().min(1, "条码类型不能为空"),
    status: z.coerce.number().min(0, "状态必须大于等于0"),
    image_format: z.string().min(1, "图片格式不能为空"),
    image_width: z.coerce.number().min(1, "图片宽度必须大于0"),
    image_height: z.coerce.number().min(1, "图片高度必须大于0"),
    margin: z.coerce.number().min(0, "边距不能小于0"),
    image_color: z.string().regex(/^#[0-9A-Fa-f]{6}$/, "前景色格式错误，应为 #RRGGBB"),
    image_background: z.string().regex(/^#[0-9A-Fa-f]{6}$/, "背景色格式错误，应为 #RRGGBB"),
});

export type BarcodeCreateConfigFormType = z.infer<typeof BarcodeCreateConfigFormSchema>;
