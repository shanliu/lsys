/**
 * authDownloadFetch
 *
 * 统一处理文件下载 POST 请求：
 *   - baseURL 来自 Config.apiBaseUrl，与其他 API 保持一致
 *   - 自动注入 Authorization Bearer token
 *   - 返回原生 Response（保留二进制流，不经过 JSON 解析）
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
