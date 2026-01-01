import { PageDataParam } from '@shared/types/base-schema';
import { z } from "zod";

// 基础过滤器字段 schema
const BarcodeParseRecordFilterBaseSchema = z.object({
    barcode_type: z.string().optional(),
    status: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const BarcodeParseRecordFilterParamSchema = BarcodeParseRecordFilterBaseSchema.extend(PageDataParam);

// 表单 schema
export const BarcodeParseRecordFilterFormSchema = z.object({
    barcode_type: z.string().optional(),
    status: z.string().optional(),
});

export type BarcodeParseRecordFilterParamType = z.infer<typeof BarcodeParseRecordFilterParamSchema>;
