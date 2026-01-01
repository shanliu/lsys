import { userStore } from "@shared/lib/auth";


export const userQueryKey = (() => {
    let cachedFingerprint = "anon";
    let lastBearer = "";
    let lastUserId = 0;

    return (...parts: unknown[]) => {
        const currentUser = userStore.getState().current?.();
        const bearer = currentUser?.bearer ?? "";
        const userId = currentUser?.userId ?? 0;

        if (bearer !== lastBearer || userId !== lastUserId) {
            if (bearer && userId > 0) {
                // 使用整个 bearer 进行 CRC32 计算
                let crc = 0xffffffff >>> 0;
                for (let i = 0; i < bearer.length; i++) {
                    crc = (crc ^ bearer.charCodeAt(i)) >>> 0;
                    for (let j = 0; j < 8; j++) {
                        crc = crc & 1 ? (crc >>> 1) ^ 0xedb88320 : crc >>> 1;
                    }
                }
                crc = ~crc >>> 0;

                // UID 在前，CRC32 转字符串作为指纹
                cachedFingerprint = `${userId}_${crc.toString(36)}`;
            } else {
                cachedFingerprint = "anon";
            }
            lastBearer = bearer;
            lastUserId = userId;
        }

        return [cachedFingerprint, ...parts] as const;
    };
})();

/**
 * 生成 App 查询的 queryKey
 * @param appId 应用 ID
 * @param attrs 可选的属性参数，用于区分不同的查询结果
 * @returns 返回包含用户指纹和属性参数的 queryKey 数组
 */
export function appQueryKey(
    appId: string | number,
    attrs?: {
        attr_inner_feature?: boolean;
        attr_oauth_client_data?: boolean;
        attr_parent_app?: boolean;
        attr_sub_app_count?: boolean;
        attr_exter_feature?: boolean;
    }
) {
    const key = ['app-detail', appId];
    if (attrs) {
        key.push(JSON.stringify(attrs));
    }
    return userQueryKey(...key);
}
