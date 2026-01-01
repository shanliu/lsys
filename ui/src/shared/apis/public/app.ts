import { baseApi } from "@shared/lib/apis/api_base";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// Public App APIs


// 地区列表参数
export const appBarcodeShowParamSchema = z.object({
    /** 行政区域编码 */
    //  contentType: string, // 'text' | 'base64'
    contentType: z.string(),
    codeId: z.coerce.number(), // 二维码配置ID
    contentData: z.string(), // 二维码内容
});
export type appBarcodeShowParamType = z.infer<typeof appBarcodeShowParamSchema>;

/**
 * 显示二维码
 * @description 根据内容类型、配置ID和内容数据生成二维码图片
 * URL格式：/barcode/{content_type}/{code_id}/{content_data}
 */
export async function appBarcodeShow(
    param: appBarcodeShowParamType,
    config?: AxiosRequestConfig<any>
): Promise<Blob> {
    const response = await baseApi().get(
        `/barcode/${param.contentType}/${param.codeId}/${encodeURIComponent(param.contentData)}`,
        {
            ...config,
            responseType: 'blob', // 返回图片数据
        }
    );
    return response.data;
}


