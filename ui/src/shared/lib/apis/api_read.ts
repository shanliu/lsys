/**
 * api_read
 *
 * 文件读取相关工具函数：
 *   - authDownloadFetch：统一处理文件下载 POST 请求（注入 Bearer token）
 *   - getAppFileShareUrl：生成公开文件的分享 URL（GET /api/user/app_file/share/{key}）
 *   - getUserFileShareUrl：生成用户文件的分享 URL（GET /api/user/file/share/{key}）
 */
import { userStore } from "@shared/lib/auth";
import { Config } from "../config";

export async function authDownloadFetch(
    path: string,
    body: Record<string, unknown> = {}
): Promise<Response> {
    const base = Config.apiBaseUrl.replace(/\/+$/, "");
    const url = path.startsWith("http") ? path : `${base}${path}`;

    const headers: Record<string, string> = {
        "Content-Type": "application/json",
    };
    const bearer = userStore.getState().current()?.bearer;
    if (bearer) headers["Authorization"] = `Bearer ${bearer}`;

    return fetch(url, {
        method: "POST",
        headers,
        body: JSON.stringify(body),
    });
}

/**
 * 生成应用文件的公开分享 URL
 * 对应后端路由：GET /api/user/app_file/share/{key}
 */
export function getAppFileShareUrl(key: string): string {
    const base = Config.apiBaseUrl.replace(/\/+$/, "");
    return `${base}/api/user/app_file/share/${encodeURIComponent(key)}`;
}

/**
 * 生成用户文件的公开分享 URL
 * 对应后端路由：GET /api/user/file/share/{key}
 */
export function getUserFileShareUrl(key: string): string {
    const base = Config.apiBaseUrl.replace(/\/+$/, "");
    return `${base}/api/user/file/share/${encodeURIComponent(key)}`;
}
