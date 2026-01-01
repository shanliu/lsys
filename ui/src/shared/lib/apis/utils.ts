import FingerprintJS from "@fingerprintjs/fingerprintjs";
import { ApiConfig, ApiResult, ApiResultConfig, ApiResultParse } from "@shared/types/apis-rest";
import axios from "axios";
import { parse } from "json-bigint";
import { z } from "zod";




let deviceIdPromise: Promise<string | undefined> | null = null;

const resolveDeviceId = () => {
  if (typeof window === "undefined") {
    return Promise.resolve(undefined);
  }
  if (!deviceIdPromise) {
    deviceIdPromise = FingerprintJS.load()
      .then((fp) => fp.get())
      .then((result) => result.visitorId)
      .catch(() => undefined);
  }
  return deviceIdPromise;
};

function isServiceAuthConfig(server: ApiConfig | ApiResultConfig): server is ApiResultConfig {
  return 'parseResult' in server &&
    typeof server.parseResult === 'function' &&
    'parseErrorResult' in server &&
    typeof server.parseErrorResult === 'function';
}



export const createApiResultParse = (
  statusCheck: (data: any) => boolean
): ApiResultParse => {
  return {
    parseResult: function (data: any): ApiResult {
      let message = data?.result?.message;
      if (message === undefined) {
        if (typeof data === "object" && data !== null) {
          try {
            message = JSON.stringify(data);
          } catch {
            message = String(data);
          }
        } else {
          message = String(data);
        }
      }
      return {
        status: statusCheck(data),
        code: data?.result?.code || '500',
        state: data?.result?.state || 'error',
        message,
        response: data.response || null
      };
    },
    parseErrorResult: function (error: any): ApiResult {
      let message = error?.response?.data?.result?.message;
      if (!message) {
        if (typeof error?.response?.data === "object") {
          try {
            message = JSON.stringify(error?.response?.data);
          } catch { }
        }
        if (typeof error?.response?.data === "string") {
          message = error?.response?.data;
        }
      }
      if (!message) {
        message = error?.message || error?.code || '请求错误'
      }
      return {
        status: false,
        code: error?.response?.status || error?.status || '500',
        state: error?.response?.data?.result?.state || error?.name || 'error',
        message: message,
        response: null
      };
    }
  };
};


export const createApiClient = (server: ApiConfig | ApiResultConfig) => {
  const client = axios.create({
    baseURL: server.apiBaseUrl,
    timeout: server.timeout,
    transformResponse: [
      (data: any) => {
        try {
          return parse(data)
        } catch {
          return data
        }
      }
    ]
  });

  client.interceptors.request.use(async (config) => {
    const headers = server.headers(config) || {};
    for (const key of Object.keys(headers)) {
      config.headers.set(key, headers[key]);
    }
    const deviceId = await resolveDeviceId();
    if (deviceId) {
      config.headers.set('X-Device-ID', deviceId);
    }
    return config;
  });
  client.interceptors.response.use(
    (response) => {
      response.data = server.parseData(response.data);
      if (isServiceAuthConfig(server)) {
        response.data = server.parseResult(response.data);
        if (response.data.status)
          return Promise.resolve(response);
        else
          return Promise.reject(response);
      }
      return response;
    },
    (error) => {
      if (isServiceAuthConfig(server)) {
        if (typeof error === 'object') {
          error.data = server.parseErrorResult(error);
        } else {
          error = {
            status: "500",
            error: error,
            data: server.parseErrorResult(error),
          }
        }
      }
      throw error;
    }
  );
  return client;
};



export function parseResData<T>(data: any, schema: z.ZodSchema<T>): Promise<ApiResult<T>> {
  if (data?.status && data?.response) {
    const parseResult = schema.safeParse(data.response);
    if (parseResult.success) {
      data.response = parseResult.data;
      return Promise.resolve(data);
    } else {
      data.status = false;
      data.message = 'zod parse:' + parseResult.error.errors.map((e) => JSON.stringify(e)).join(", ");
      data.response = undefined;
      return Promise.reject(data);
    }
  }
  return Promise.reject(data);
}

/**
 * 清理参数对象中的空字符串属性
 * 用于列表接口过滤参数，当可选字符串参数为空字符串时删除该参数
 * @param param 参数对象
 * @param keys 需要检查的属性名数组
 * @returns 清理后的参数对象
 */
export function cleanEmptyStringParams<T extends Record<string, any>>(
  param: T,
  keys: (keyof T)[]
): T {
  const result = { ...param };
  for (const key of keys) {
    if (result[key] === "") {
      delete result[key];
    }
  }
  return result;
}
