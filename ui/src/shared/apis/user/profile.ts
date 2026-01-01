import { authApi } from "@shared/lib/apis/api_auth";
import { parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { CaptChaParam, MobileSchema, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

/**
 * 用户资料管理 API
 * 对应文档: docs/api/user/account_profile/
 * 用户个人资料信息管理，包括手机号、邮箱、地址、外部账号绑定等
 */

/**
 * 手机号管理 API
 */

// 手机号数据结构 - 根据文档调整
export const MobileDataSchema = z.object({
    /** 手机号ID */
    id: z.coerce.number(),
    /** 账号ID */
    account_id: z.coerce.number(),
    /** 区号 */
    area_code: z.string(),
    /** 手机号 */
    mobile: z.string(),
    /** 验证状态: 1-待验证, 2-已验证 */
    status: z.coerce.number(),
    /** 修改时间 */
    change_time: UnixTimestampSchema,
    /** 确认时间 */
    confirm_time: UnixTimestampSchema,
});
export type MobileDataType = z.infer<typeof MobileDataSchema>;

export const MobileListDataResSchema = z.object({
    data: z.array(MobileDataSchema),
    total: z.string(),
});
export type MobileListDataResType = z.infer<typeof MobileListDataResSchema>;

// 添加手机号参数
export const MobileListDataParamSchema = z.object({
    status: z.number().optional(),
});
export type MobileListDataParamType = z.infer<typeof MobileListDataParamSchema>;

/**
 * 获取手机号列表
 * @description 获取当前用户的手机号列表信息
 */
export async function mobileListData(
    param: MobileListDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<MobileListDataResType>> {
    const { data } = await authApi().post("/api/user/profile/mobile/list_data", param, config);
    return parseResData(data, MobileListDataResSchema);
}

// 添加手机号参数
export const MobileAddParamSchema = z.object({
    /** 区号 */
    area_code: z.string().min(1, "区号不能为空"),
    /** 手机号 */
    mobile: MobileSchema,
});
export type MobileAddParamType = z.infer<typeof MobileAddParamSchema>;

export const MobileAddResSchema = z.object({
    /** 手机号ID */
    id: z.coerce.number(),
});
export type MobileAddResType = z.infer<typeof MobileAddResSchema>;

/**
 * 添加手机号
 * @description 为用户添加新的手机号，添加后需要验证
 */
export async function mobileAdd(
    param: MobileAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<MobileAddResType>> {
    const { data } = await authApi().post("/api/user/profile/mobile/add", param, config);
    return parseResData(data, MobileAddResSchema);
}

// 删除手机号参数
export const MobileDeleteParamSchema = z.object({
    /** 手机号ID */
    mobile_id: z.coerce.number().min(1, "手机号ID不能为空"),
});
export type MobileDeleteParamType = z.infer<typeof MobileDeleteParamSchema>;

/**
 * 删除手机号
 * @description 删除用户的指定手机号
 */
export async function mobileDelete(
    param: MobileDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/profile/mobile/delete", param, config);
    return data;
}

// 发送手机验证码参数
export const MobileSendCodeParamSchema = z.object({
    /** 区号 */
    area_code: z.string().min(1, "区号不能为空"),
    /** 手机号 */
    mobile: MobileSchema,
    ...CaptChaParam
});
export type MobileSendCodeParamType = z.infer<typeof MobileSendCodeParamSchema>;

/**
 * 发送手机验证码
 * @description 向指定手机号发送验证码用于验证
 */
export async function mobileSendCode(
    param: MobileSendCodeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/profile/mobile/send_code", param, config);
    return data;
}

// 确认手机验证码参数
export const MobileConfirmParamSchema = z.object({
    /** 手机号ID */
    mobile_id: z.coerce.number().min(1, "手机号ID不能为空"),
    /** 验证码 */
    code: z.string().min(1, "验证码不能为空"),
});
export type MobileConfirmParamType = z.infer<typeof MobileConfirmParamSchema>;

/**
 * 确认手机验证码
 * @description 提交手机验证码完成手机号验证
 */
export async function mobileConfirm(
    param: MobileConfirmParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/profile/mobile/confirm", param, config);
    return data;
}

/**
 * 邮箱管理 API
 */

// 邮箱数据结构 - 根据文档调整
export const EmailDataSchema = z.object({
    /** 邮箱ID */
    id: z.coerce.number(),
    /** 账号ID */
    account_id: z.coerce.number(),
    /** 邮箱地址 */
    email: z.string(),
    /** 验证状态: 1-待验证, 2-已验证 */
    status: z.coerce.number(),
    /** 修改时间 */
    change_time: UnixTimestampSchema,
    /** 确认时间 */
    confirm_time: UnixTimestampSchema,
});
export type EmailDataType = z.infer<typeof EmailDataSchema>;

export const EmailListDataResSchema = z.object({
    data: z.array(EmailDataSchema),
    total: z.string(),
});
export type EmailListDataResType = z.infer<typeof EmailListDataResSchema>;
export const EmailListDataParamSchema = z.object({
    status: z.number().optional(),
});
export type EmailListDataParamType = z.infer<typeof EmailListDataParamSchema>;
/**
 * 获取邮箱列表
 * @description 获取当前用户的邮箱列表信息
 */
export async function emailListData(
    param: EmailListDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<EmailListDataResType>> {
    const { data } = await authApi().post("/api/user/profile/email/list_data", param, config);
    return parseResData(data, EmailListDataResSchema);
}

// 添加邮箱参数
export const EmailAddParamSchema = z.object({
    /** 邮箱地址 */
    email: z.string().email("请输入正确的邮箱地址"),
});
export type EmailAddParamType = z.infer<typeof EmailAddParamSchema>;

export const EmailAddResSchema = z.object({
    /** 邮箱ID */
    id: z.coerce.number(),
});
export type EmailAddResType = z.infer<typeof EmailAddResSchema>;

/**
 * 添加邮箱
 * @description 为用户添加新的邮箱地址，添加后需要验证
 */
export async function emailAdd(
    param: EmailAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<EmailAddResType>> {
    const { data } = await authApi().post("/api/user/profile/email/add", param, config);
    return parseResData(data, EmailAddResSchema);
}

// 删除邮箱参数
export const EmailDeleteParamSchema = z.object({
    /** 邮箱ID */
    email_id: z.coerce.number().min(1, "邮箱ID不能为空"),
});
export type EmailDeleteParamType = z.infer<typeof EmailDeleteParamSchema>;

/**
 * 删除邮箱
 * @description 删除用户的指定邮箱
 */
export async function emailDelete(
    param: EmailDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/profile/email/delete", param, config);
    return data;
}

// 发送邮箱验证码参数
export const EmailSendCodeParamSchema = z.object({
    /** 邮箱地址 */
    email: z.string().email("请输入正确的邮箱地址"),
    ...CaptChaParam
});
export type EmailSendCodeParamType = z.infer<typeof EmailSendCodeParamSchema>;

/**
 * 发送邮箱验证码
 * @description 向指定邮箱发送验证码用于验证
 */
export async function emailSendCode(
    param: EmailSendCodeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/profile/email/send_code", param, config);
    return data;
}

// 确认邮箱验证码参数
export const EmailConfirmParamSchema = z.object({
    /** 邮箱ID */
    email_id: z.coerce.number().min(1, "邮箱ID不能为空"),
    /** 验证码 */
    code: z.string().min(1, "验证码不能为空"),
});
export type EmailConfirmParamType = z.infer<typeof EmailConfirmParamSchema>;

/**
 * 确认邮箱验证码
 * @description 提交邮箱验证码完成邮箱验证
 */
export async function emailConfirm(
    param: EmailConfirmParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/profile/email/confirm", param, config);
    return data;
}

/**
 * 地址管理 API
 */

// 地区详情数据项
export const AddressCodeDetailItemSchema = z.object({
    /** 地区编码 */
    code: z.string(),
    /** 地区名称 */
    name: z.string(),
});
export type AddressCodeDetailItemType = z.infer<typeof AddressCodeDetailItemSchema>;

// 地址数据结构 - 根据文档调整
export const AddressDataSchema = z.object({
    /** 地址ID */
    id: z.coerce.number(),
    /** 地区编码 */
    address_code: z.string(),
    /** 详细地址 */
    address_detail: z.string(),
    /** 地区信息 */
    address_info: z.string(),
    /** 修改时间 */
    change_time: UnixTimestampSchema,
    /** 地区详情 */
    code_detail: z.array(AddressCodeDetailItemSchema),
    /** 国家编码 */
    country_code: z.string(),
    /** 手机号码 */
    mobile: z.string(),
    /** 收件人姓名 */
    name: z.string(),
});
export type AddressDataType = z.infer<typeof AddressDataSchema>;

export const AddressListDataResSchema = z.object({
    data: z.array(AddressDataSchema),
    total: z.string(),
});
export type AddressListDataResType = z.infer<typeof AddressListDataResSchema>;

/**
 * 获取地址列表
 * @description 获取当前用户的地址列表信息
 */
export async function addressListData(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AddressListDataResType>> {
    const { data } = await authApi().post("/api/user/profile/address/list_data", {}, config);
    return parseResData(data, AddressListDataResSchema);
}

// 添加地址参数 - 根据文档调整
export const AddressAddParamSchema = z.object({
    /** 地区编码 */
    code: z.string().min(1, "地区编码不能为空"),
    /** 地区信息 */
    info: z.string().min(1, "地区信息不能为空"),
    /** 详细地址 */
    detail: z.string().min(1, "详细地址不能为空"),
    /** 收件人姓名 */
    name: z.string().min(1, "收件人姓名不能为空"),
    /** 手机号码 */
    mobile: MobileSchema,
});
export type AddressAddParamType = z.infer<typeof AddressAddParamSchema>;

export const AddressAddResSchema = z.object({
    /** 地址ID */
    id: z.coerce.number(),
});
export type AddressAddResType = z.infer<typeof AddressAddResSchema>;

/**
 * 添加地址
 * @description 为用户添加新的地址信息
 */
export async function addressAdd(
    param: AddressAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AddressAddResType>> {
    const { data } = await authApi().post("/api/user/profile/address/add", param, config);
    return parseResData(data, AddressAddResSchema);
}

// 编辑地址参数 - 根据文档调整
export const AddressEditParamSchema = z.object({
    /** 地址ID */
    address_id: z.coerce.number().min(1, "地址ID不能为空"),
    /** 地区编码 */
    code: z.string().min(1, "地区编码不能为空"),
    /** 地区信息 */
    info: z.string().min(1, "地区信息不能为空"),
    /** 详细地址 */
    detail: z.string().min(1, "详细地址不能为空"),
    /** 收件人姓名 */
    name: z.string().min(1, "收件人姓名不能为空"),
    /** 手机号码 */
    mobile: MobileSchema,
});
export type AddressEditParamType = z.infer<typeof AddressEditParamSchema>;

/**
 * 编辑地址
 * @description 修改用户的地址信息
 */
export async function addressEdit(
    param: AddressEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/profile/address/edit", param, config);
    return data;
}

// 删除地址参数 - 根据文档调整
export const AddressDeleteParamSchema = z.object({
    /** 地址ID */
    address_id: z.coerce.number().min(1, "地址ID不能为空"),
});
export type AddressDeleteParamType = z.infer<typeof AddressDeleteParamSchema>;

/**
 * 删除地址
 * @description 删除用户的指定地址
 */
export async function addressDelete(
    param: AddressDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/profile/address/delete", param, config);
    return data;
}

/**
 * 外部账号绑定 API
 */

// 外部账号数据结构 - 根据文档调整
export const ExternalAccountDataSchema = z.object({
    /** 外部账号ID */
    id: z.coerce.number(),
    /** 账号ID */
    account_id: z.coerce.number(),
    /** 配置名称 */
    config_name: z.string(),
    /** 外部账号类型 */
    external_type: z.string(),
    /** 外部账号标识 */
    external_id: z.string(),
    /** 外部账号昵称 - 注意：接口返回字段为 nikename */
    external_nikename: z.string(),
    /** 外部账号名称 */
    external_name: z.string(),
    /** 外部账号性别 */
    external_gender: z.string(),
    /** 外部账号链接 */
    external_link: z.string(),
    /** 外部账号图片 */
    external_pic: z.string(),
    /** 绑定状态 */
    status: z.coerce.number(),
    /** 修改时间 */
    change_time: UnixTimestampSchema,
    /** Token 数据 */
    token_data: z.string(),
    /** Token 过期时间 */
    token_timeout: z.coerce.number(),
});
export type ExternalAccountDataType = z.infer<typeof ExternalAccountDataSchema>;

export const ExternalListDataResSchema = z.object({
    data: z.array(ExternalAccountDataSchema),
    total: z.string(),
});
export type ExternalListDataResType = z.infer<typeof ExternalListDataResSchema>;


export const ExternalListDataParamSchema = z.object({
    oauth_type: z.string().nullable().optional(),
});
export type ExternalListDataParamType = z.infer<typeof ExternalListDataParamSchema>;

/**
 * 获取外部账号绑定列表
 * @description 获取当前用户绑定的外部账号列表
 */
export async function externalListData(
    param: ExternalListDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<ExternalListDataResType>> {
    const { data } = await authApi().post("/api/user/profile/exter/list_data", param || {}, config);
    return parseResData(data, ExternalListDataResSchema);
}

// 获取外部账号绑定URL参数 - 根据文档调整
export const ExternalBindUrlParamSchema = z.object({
    /** 外部账号类型 */
    login_type: z.string().min(1, "外部账号类型不能为空"),
    /** 登录状态标识 */
    login_state: z.string().min(1, "登录状态标识不能为空"),
    /** 回调URL */
    callback_url: z.string().url("请输入正确的回调URL"),
});
export type ExternalBindUrlParamType = z.infer<typeof ExternalBindUrlParamSchema>;

export const ExternalBindUrlResSchema = z.object({
    /** 绑定URL */
    url: z.string(),
});
export type ExternalBindUrlResType = z.infer<typeof ExternalBindUrlResSchema>;

/**
 * 获取外部账号绑定URL
 * @description 获取用于绑定外部账号的授权URL
 */
export async function externalBindUrl(
    param: ExternalBindUrlParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<ExternalBindUrlResType>> {
    const { data } = await authApi().post("/api/user/profile/exter/bind_url", param, config);
    return parseResData(data, ExternalBindUrlResSchema);
}

// 检查外部账号绑定参数 - 根据文档调整
export const ExternalBindCheckParamSchema = z.object({
    /** 外部账号类型 */
    login_type: z.string().min(1, "外部账号类型不能为空"),
    /** 登录状态标识 */
    login_state: z.string().min(1, "登录状态标识不能为空"),
});
export type ExternalBindCheckParamType = z.infer<typeof ExternalBindCheckParamSchema>;

export const ExternalBindCheckResSchema = z.object({
    /** 是否需要重试: 1-继续重试 */
    reload: z.string().optional(),
    /** 绑定成功的ID */
    id: z.string().optional(),
});
export type ExternalBindCheckResType = z.infer<typeof ExternalBindCheckResSchema>;

/**
 * 检查外部账号绑定状态
 * @description 检查外部账号绑定是否完成
 */
export async function externalBindCheck(
    param: ExternalBindCheckParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<ExternalBindCheckResType>> {
    const { data } = await authApi().post("/api/user/profile/exter/bind_check", param, config);
    return parseResData(data, ExternalBindCheckResSchema);
}

// 删除外部账号绑定参数 - 根据文档调整
export const ExternalDeleteParamSchema = z.object({
    /** 外部账号ID */
    ext_id: z.coerce.number().min(1, "外部账号ID不能为空"),
});
export type ExternalDeleteParamType = z.infer<typeof ExternalDeleteParamSchema>;

/**
 * 删除外部账号绑定
 * @description 解绑用户的外部账号
 */
export async function externalDelete(
    param: ExternalDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/profile/exter/delete", param, config);
    return data;
}


