import { type AppRequestItemType } from "@shared/apis/admin/app"

/**
 * 检查是否有请求数据
 */
export function hasRequestData(data: AppRequestItemType): boolean {
    const { request_type, change_data, feature_data, oauth_client_data } = data

    // 类型 3, 4, 5 无附带数据
    if (request_type === 3 || request_type === 4 || request_type === 5) {
        return false
    }

    // 类型 1, 2 检查 change_data
    if ((request_type === 1 || request_type === 2) && change_data && Object.keys(change_data).length > 0) {
        return true
    }

    // 类型 8 检查 feature_data
    if (request_type === 8 && feature_data && Object.keys(feature_data).length > 0) {
        return true
    }

    // 类型 6, 7 检查 oauth_client_data
    if ((request_type === 6 || request_type === 7) && oauth_client_data && Object.keys(oauth_client_data).length > 0) {
        return true
    }

    return false
}
