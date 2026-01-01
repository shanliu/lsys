import { baseApi } from "@shared/lib/apis/api_base";
import { parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { AxiosRequestConfig } from "axios";
import z from "zod";

/**
 * 地区相关 API
 * 对应文档: docs/api/public/area/
 */

// 地区数据结构
export const AreaDataSchema = z.object({
    /** 区域编码 */
    code: z.string(),
    /** 区域名称 */
    name: z.string(),
    /** 是否是叶子节点(0:否,1:是) */
    leaf: z.string(),
});
export type AreaDataType = z.infer<typeof AreaDataSchema>;

// 地区列表参数
export const AreaListParamSchema = z.object({
    /** 行政区域编码 */
    code: z.string().min(1, "区域编码不能为空"),
});
export type AreaListParamType = z.infer<typeof AreaListParamSchema>;

export const AreaListResSchema = z.object({
    /** 区域列表 */
    area: z.array(AreaDataSchema),
});
export type AreaListResType = z.infer<typeof AreaListResSchema>;

/**
 * 根据行政区域编码获取下级区域列表
 * @description 获取指定区域编码下的下级行政区域列表，如省下的市、市下的区县等
 */
export async function areaList(
    param: AreaListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AreaListResType>> {
    const { data } = await baseApi().post("/api/area/list", param, config);
    return parseResData(data, AreaListResSchema);
}

// 地区搜索参数 - 根据文档修正
export const AreaSearchParamSchema = z.object({
    /** 搜索关键词 */
    key_word: z.string().min(1, "搜索关键词不能为空"),
    limit: z.coerce.number().min(1).max(100).default(10).optional(), // 默认值10，最大值100
});
export type AreaSearchParamType = z.infer<typeof AreaSearchParamSchema>;

// 响应结构修正为二维数组
export const AreaSearchResSchema = z.object({
    /** 搜索结果列表（二维数组，第一层为路径数组） */
    area: z.array(z.array(AreaDataSchema)),
});
export type AreaSearchResType = z.infer<typeof AreaSearchResSchema>;

/**
 * 搜索地区
 * @description 根据关键词搜索地区，返回匹配的地区及其路径
 */
export async function areaSearch(
    param: AreaSearchParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AreaSearchResType>> {
    const { data } = await baseApi().post("/api/area/search", param, config);
    return parseResData(data, AreaSearchResSchema);
}

// 地区查找参数 - 根据文档修正
export const AreaFindParamSchema = z.object({
    /** 区域编码 */
    code: z.string().min(1, "区域编码不能为空"),
});
export type AreaFindParamType = z.infer<typeof AreaFindParamSchema>;

// 响应结构修正为数组（路径）
export const AreaFindResSchema = z.object({
    /** 地区路径信息数组 */
    area: z.array(AreaDataSchema),
});
export type AreaFindResType = z.infer<typeof AreaFindResSchema>;

/**
 * 查找指定地区信息
 * @description 根据区域编码查找地区及其上级路径信息
 */
export async function areaFind(
    param: AreaFindParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AreaFindResType>> {
    const { data } = await baseApi().post("/api/area/find", param, config);
    return parseResData(data, AreaFindResSchema);
}

// 相关地区参数 - 根据文档修正
export const AreaRelatedParamSchema = z.object({
    /** 区域编码 */
    code: z.string().min(1, "区域编码不能为空"),
});
export type AreaRelatedParamType = z.infer<typeof AreaRelatedParamSchema>;

// 响应结构修正 - 二维数组，省份列表和选中省份的城市列表
export const AreaRelatedItemSchema = z.object({
    code: z.string(),
    name: z.string(),
    leaf: z.string(),
    selected: z.string(), // "0" 或 "1"
});
export type AreaRelatedItemType = z.infer<typeof AreaRelatedItemSchema>;

export const AreaRelatedResSchema = z.object({
    /** 相关地区列表（二维数组：[省份列表, 城市列表]） */
    area: z.array(z.array(AreaRelatedItemSchema)),
});
export type AreaRelatedResType = z.infer<typeof AreaRelatedResSchema>;
/**
 * 获取相关地区
 * @description 区域关联查询，返回省份列表和选中省份的城市列表
 */
export async function areaRelated(
    param: AreaRelatedParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AreaRelatedResType>> {
    const { data } = await baseApi().post("/api/area/related", param, config);
    return parseResData(data, AreaRelatedResSchema);
}

// 地理位置定位参数 - 根据文档修正
export const AreaGeoParamSchema = z.object({
    /** 纬度 */
    lat: z.coerce.number().min(-90).max(90),
    /** 经度 */
    lng: z.coerce.number().min(-180).max(180),
});
export type AreaGeoParamType = z.infer<typeof AreaGeoParamSchema>;

export const AreaGeoResSchema = z.object({
    /** 地理位置信息数组 */
    area: z.array(AreaDataSchema),
});
export type AreaGeoResType = z.infer<typeof AreaGeoResSchema>;

/**
 * 根据坐标获取地区信息
 * @description 根据经纬度坐标反向查询所在的地区信息
 */
export async function areaGeo(
    param: AreaGeoParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AreaGeoResType>> {
    const { data } = await baseApi().post("/api/area/geo", param, config);
    return parseResData(data, AreaGeoResSchema);
}

