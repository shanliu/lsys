import { AxiosInstance } from 'axios';
import { Config } from '../config';
import { createApiClient, createApiResultParse } from './utils';


export const baseApi = (): AxiosInstance => {
    const ApiParse = createApiResultParse((data: any) => {
        return data?.result?.code === "200" || data?.result?.status === "not_found";
    });
    return createApiClient({
        apiBaseUrl: Config.apiBaseUrl,
        timeout: Config.timeOut,
        headers: () => {
            const headers: Record<string, string> = {};
            headers['Content-type'] = 'application/json';
            return headers;
        },
        parseData: function (data: any) {
            return data;
        },
        parseResult: ApiParse.parseResult,
        parseErrorResult: ApiParse.parseErrorResult,
    });
}
