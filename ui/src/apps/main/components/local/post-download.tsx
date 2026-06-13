/**
 * PostDownload - POST 方式下载容器组件（表单新窗口版）
 *
 * 通过创建隐藏表单 POST 到新标签页，让浏览器原生处理下载，
 * 适用于任意大小文件，无需将文件内容缓冲到 JS 内存。
 *
 * token 从当前登录态自动附带，无需调用方传入。
 *
 * 用法示例：
 *   <PostDownload url="/api/user/export_task/download" body={{ task_id: 1 }}>
 *     {({ onClick }) => <Button onClick={onClick}>下载文件</Button>}
 *   </PostDownload>
 */

import { userStore } from "@shared/lib/auth";
import { Config } from "@shared/lib/config";
import React, { useCallback } from "react";

export interface PostDownloadRenderProps {
  /** 触发下载 */
  onClick: () => void;
  /** 始终为 false（浏览器原生处理，无 JS 加载状态） */
  isLoading: boolean;
  /** 始终为 null */
  error: string | null;
}

export interface PostDownloadProps {
  /** POST 请求 URL（相对路径或绝对路径） */
  url: string;
  /** POST 请求体（不含 token，由组件自动附带登录 token） */
  body?: Record<string, unknown>;
  /** render prop：接收下载状态，返回触发器节点 */
  children: (props: PostDownloadRenderProps) => React.ReactNode;
}

/** 取当前登录的不透明 token（lsys.<checksum>.<inner>），整串作为下载凭证回传后端 */
function getLoginToken(): string {
  return userStore.getState().current()?.bearer ?? "";
}

export function PostDownload({ url, body = {}, children }: PostDownloadProps) {
  const onClick = useCallback(() => {
    const token = getLoginToken();
    const base = Config.apiBaseUrl.replace(/\/+$/, "");
    const fullUrl = url.startsWith("http") ? url : `${base}${url}`;

    const form = document.createElement("form");
    form.method = "POST";
    form.action = fullUrl;
    form.target = "_blank";

    const fields: Record<string, unknown> = { ...body, token };
    for (const [key, value] of Object.entries(fields)) {
      const input = document.createElement("input");
      input.type = "hidden";
      input.name = key;
      input.value = String(value ?? "");
      form.appendChild(input);
    }

    document.body.appendChild(form);
    form.submit();
    form.remove();
  }, [url, body]);

  return <>{children({ onClick, isLoading: false, error: null })}</>;
}
