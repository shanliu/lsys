import { baseApi } from "@shared/lib/apis/api_base";
import { parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// Captcha API - 根据文档定义响应结构
export const CaptchaResSchema = z.object({
  image_data: z.string(), // base64编码的图片数据
  image_header: z.string(), // 图片类型，如 "image/png"
  save_time: z.coerce.number(), // 保存时间（秒）
  code_length: z.coerce.number() // 验证码长度
});
export type CaptchaResType = z.infer<typeof CaptchaResSchema>;

export const CaptchaReqSchema = z.object({
  captcha_type: z.string(), // 验证码类型，如 "login"
  captcha_tag: z.string(), // 验证码标签，自定义随机字符串
});
export type CaptchaReqType = z.infer<typeof CaptchaReqSchema>;

/**
 * 获取验证码图片
 * @param param 验证码类型
 * @param config 请求配置
 * @returns 验证码信息，包含base64图片数据
 */
export async function getCaptchaImage(
  param: CaptchaReqType,
  config?: AxiosRequestConfig<any>
): Promise<ApiResult<CaptchaResType>> {
  const { data } = await baseApi().post('/api/captcha/show', param, config);
  return parseResData(data, CaptchaResSchema);
}





export const CaptchaValidParamSchema = z.object({
  captcha_type: z.string(), // 验证码类型，如 "login"
  captcha_tag: z.string(), // 验证码标签，自定义随机字符串
  captcha_code: z.string(), // 用户输入的验证码
});
export type CaptchaValidParamType = z.infer<typeof CaptchaValidParamSchema>;

/**

 * @param config 请求配置
 * @returns 验证码信息，包含base64图片数据
 */
export async function validCaptchaData(
  param: CaptchaValidParamType,
  config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
  const { data } = await baseApi().post('/api/captcha/valid', param, config);
  return data;
}

