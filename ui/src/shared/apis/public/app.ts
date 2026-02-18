import { baseApi } from "@shared/lib/apis/api_base";
import { getBarcodeBaseUrl } from "@shared/lib/apis/api_barcode";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// Public App APIs

// ============ Barcode APIs ============

// Barcode show parameters
export const appBarcodeShowParamSchema = z.object({
    /** Content type: 'text' | 'base64' */
    contentType: z.string(),
    /** Barcode config ID */
    codeId: z.coerce.number(),
    /** Barcode content data */
    contentData: z.string(),
});
export type appBarcodeShowParamType = z.infer<typeof appBarcodeShowParamSchema>;

/**
 * Get barcode image URL
 * @description Generate URL for barcode image based on content type, config ID and content data
 * URL format: {barcodeBaseUrl}/barcode/{content_type}/{code_id}/{content_data}
 */
export function getBarcodeShowUrl(param: appBarcodeShowParamType): string {
    const baseUrl = getBarcodeBaseUrl();
    return `${baseUrl}/barcode/${param.contentType}/${param.codeId}/${encodeURIComponent(param.contentData)}`;
}

/**
 * Show barcode image
 * @description Fetch barcode image based on content type, config ID and content data
 * URL format: /barcode/{content_type}/{code_id}/{content_data}
 * @note This API now targets the independent barcode service
 */
export async function appBarcodeShow(
    param: appBarcodeShowParamType,
    config?: AxiosRequestConfig<any>
): Promise<Blob> {
    const response = await baseApi().get(
        getBarcodeShowUrl(param),
        {
            ...config,
            responseType: 'blob',
        }
    );
    return response.data;
}


