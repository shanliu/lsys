import { InternalAxiosRequestConfig } from "axios";

export interface ApiConfig {
    apiBaseUrl: string;
    timeout: number;
    headers: (req: InternalAxiosRequestConfig<any>) => null | Record<string, string>;
    parseData: (data: any) => any;
}

export interface ApiResult<D = any> {
    code: string;
    state: string;
    status: boolean;
    message: string;
    response?: D;
}


export interface ApiResultParse {
    parseResult: (data: any) => ApiResult;
    parseErrorResult: (error: any) => ApiResult;
}

export interface ApiResultConfig extends ApiConfig, ApiResultParse { }

