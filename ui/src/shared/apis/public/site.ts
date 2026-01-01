import { baseApi } from "@shared/lib/apis/api_base";
import { parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// Site Info API - 根据文档修正响应结构
export const SiteInfoResSchema = z.object({
    exter_type: z.array(z.string()), // 外部登录支持类型列表
    site_tips: z.string(), // 站点提示信息
});
export type SiteInfoResType = z.infer<typeof SiteInfoResSchema>;

export async function siteInfo(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SiteInfoResType>> {
    const { data } = await baseApi().post('/api/site/info', {}, config);
    return parseResData(data, SiteInfoResSchema);
}


