/**
 * 通用 SSE（Server-Sent Events）Hook
 *
 * 使用 fetch + ReadableStream 实现，支持 POST 请求体和自定义请求头。
 * 不包含具体业务数据解析，仅负责连接管理和原始文本事件的分发。
 *
 * 特性：
 * - 支持 GET / POST
 * - 支持自定义请求头（含 Authorization）
 * - 自动解析 SSE `data:` 行，每条消息调用 onMessage
 * - enabled=false 或 disconnect() 时自动断开
 * - 组件卸载时自动清理
 */

import { useCallback, useEffect, useRef, useState } from "react";

export type SseStatus = "idle" | "connecting" | "connected" | "closed" | "error";

export interface UseSseOptions {
  /** 请求地址 */
  url: string;
  /** HTTP 方法，默认 "POST" */
  method?: "GET" | "POST";
  /** POST 请求体（序列化后的 JSON 字符串，或 null/undefined） */
  body?: string | null;
  /**
   * 返回请求头的函数（每次建立连接时调用，确保 token 最新）
   * 不需要设置 Content-Type，会自动设置
   */
  getHeaders?: () => Record<string, string>;
  /** 收到消息时触发，data 为 SSE `data:` 字段的原始内容 */
  onMessage: (data: string) => void;
  /** 连接成功打开时触发 */
  onOpen?: () => void;
  /** 流正常结束（服务端关闭）时触发 */
  onComplete?: () => void;
  /** 发生错误时触发 */
  onError?: (error: Error) => void;
  /** 是否启用，false 时不建立连接，默认 true */
  enabled?: boolean;
}

export interface UseSseReturn {
  status: SseStatus;
  error: Error | null;
  /** 手动重新连接（会先断开当前连接） */
  reconnect: () => void;
  /** 手动断开连接 */
  disconnect: () => void;
}

export function useSse(options: UseSseOptions): UseSseReturn {
  const {
    url,
    method = "POST",
    body,
    getHeaders,
    onMessage,
    onOpen,
    onComplete,
    onError,
    enabled = true,
  } = options;

  const [status, setStatus] = useState<SseStatus>("idle");
  const [error, setError] = useState<Error | null>(null);

  // 使用 ref 存储最新回调，避免 effect 重建
  const onMessageRef = useRef(onMessage);
  const onOpenRef = useRef(onOpen);
  const onCompleteRef = useRef(onComplete);
  const onErrorRef = useRef(onError);
  // getHeaders 也存 ref：每次连接时取最新 token，但不作为 connect 的依赖，
  // 避免因父组件每次 render 传入新函数引用导致 connect 重建 → 无限重连
  const getHeadersRef = useRef(getHeaders);
  onMessageRef.current = onMessage;
  onOpenRef.current = onOpen;
  onCompleteRef.current = onComplete;
  onErrorRef.current = onError;
  getHeadersRef.current = getHeaders;

  // AbortController ref，用于取消 fetch
  const abortRef = useRef<AbortController | null>(null);
  // 是否主动断开（区分主动关闭和网络错误）
  const isManualDisconnectRef = useRef(false);

  const connect = useCallback(() => {
    // 取消已有连接
    if (abortRef.current) {
      isManualDisconnectRef.current = true;
      abortRef.current.abort();
    }

    const controller = new AbortController();
    abortRef.current = controller;
    isManualDisconnectRef.current = false;

    setStatus("connecting");
    setError(null);

    const headers: Record<string, string> = {
      Accept: "text/event-stream",
      "Cache-Control": "no-cache",
      ...(method === "POST" ? { "Content-Type": "application/json" } : {}),
      ...(getHeadersRef.current?.() ?? {}),
    };

    fetch(url, {
      method,
      headers,
      body: method === "POST" ? (body ?? null) : undefined,
      signal: controller.signal,
    })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`SSE 连接失败：HTTP ${response.status}`);
        }
        if (!response.body) {
          throw new Error("SSE 响应体为空");
        }

        setStatus("connected");
        onOpenRef.current?.();

        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = "";

        try {
          while (true) {
            const { done, value } = await reader.read();
            if (done) break;

            buffer += decoder.decode(value, { stream: true });
            // 按 SSE 协议以空行分隔事件，避免网络分片导致的半包/粘包解析问题。
            while (true) {
              const separatorIndex = buffer.indexOf("\n\n");
              if (separatorIndex === -1) break;

              const rawEvent = buffer.slice(0, separatorIndex);
              buffer = buffer.slice(separatorIndex + 2);

              const dataLines: string[] = [];
              for (const rawLine of rawEvent.split("\n")) {
                const line = rawLine.replace(/\r$/, "");
                if (line.startsWith(":")) continue;
                if (line.startsWith("data:")) {
                  dataLines.push(line.slice(5).trimStart());
                }
                // 忽略 event:, id:, retry: 等其他 SSE 字段
              }

              if (dataLines.length > 0) {
                onMessageRef.current(dataLines.join("\n"));
              }
            }
          }
        } finally {
          reader.releaseLock();
        }

        // 流正常结束
        if (!isManualDisconnectRef.current) {
          setStatus("closed");
          onCompleteRef.current?.();
        }
      })
      .catch((err: unknown) => {
        if (isManualDisconnectRef.current) {
          setStatus("idle");
          return;
        }
        const error =
          err instanceof Error ? err : new Error(String(err));
        // AbortError 是主动中断，不算错误
        if (error.name === "AbortError") {
          setStatus("idle");
          return;
        }
        setStatus("error");
        setError(error);
        onErrorRef.current?.(error);
      });
  }, [url, method, body]); // eslint-disable-line react-hooks/exhaustive-deps

  const disconnect = useCallback(() => {
    if (abortRef.current) {
      isManualDisconnectRef.current = true;
      abortRef.current.abort();
      abortRef.current = null;
    }
    setStatus("idle");
  }, []);

  const reconnect = useCallback(() => {
    connect();
  }, [connect]);

  useEffect(() => {
    if (!enabled) {
      disconnect();
      return;
    }
    connect();
    return () => {
      if (abortRef.current) {
        isManualDisconnectRef.current = true;
        abortRef.current.abort();
        abortRef.current = null;
      }
    };
  }, [enabled, connect]); // eslint-disable-line react-hooks/exhaustive-deps

  return { status, error, reconnect, disconnect };
}
