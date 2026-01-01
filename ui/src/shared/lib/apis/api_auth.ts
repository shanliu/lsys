import { userStore } from '@shared/lib/auth';
import { AxiosInstance } from 'axios';
import { Config } from '../config';
import { createApiClient, createApiResultParse } from './utils';

export const authApi = (): AxiosInstance => {
    const ApiParse = createApiResultParse((data: any) => {
        if (data?.result?.state === "not_login" ||
            data?.result?.state === "jwt_bad_token" ||
            data?.result?.state === "jwt_parse_system"
        ) {
            const state = userStore.getState();
            const msg = data?.result?.message || data?.result?.state;
            state.invalidatedUser(state.useUserId, msg);
        }
        return data?.result?.code === "200" || data?.result?.state === "not_found";
    });
    return createApiClient({
        apiBaseUrl: Config.apiBaseUrl,
        timeout: Config.timeOut,
        headers: () => {
            let headers: Record<string, string> = {};
            headers['Content-type'] = 'application/json';
            if (!headers['Authorization']) {
                const loginData = userStore.getState().current()
                if (loginData) {
                    headers['Authorization'] = 'Bearer ' + loginData.bearer;
                };
            };
            return headers;
        },
        parseData: function (data: any) {
            return data;
        },
        parseResult: ApiParse.parseResult,
        parseErrorResult: ApiParse.parseErrorResult,
    });
}
