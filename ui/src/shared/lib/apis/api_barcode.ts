/**
 * Barcode Service API Client
 * 
 * This module provides API client for the barcode HTTP service running in the same domain.
 * 
 * JWT is obtained from userStore (same as other APIs) and used to authenticate
 * requests to the barcode service. Requests are directed to the barcode service URL
 * configured in BARCODE_BASE_URL while using the current user's JWT token.
 */

import { Config } from '../config';
import { userStore } from '../auth';
import { createApiClient, createApiResultParse } from './utils';
import { AxiosInstance } from 'axios';

/**
 * Get barcode service base URL
 * @throws Error if barcodeUrl is not configured
 */
export function getBarcodeBaseUrl(): string {
    if (!Config.appHost.barcodeUrl) {
        throw new Error('Barcode service URL (BARCODE_BASE_URL) is not configured');
    }
    return Config.appHost.barcodeUrl;
}

/**
 * Create API client for barcode service
 * Uses JWT from current user (userStore) to authenticate requests
 * Directs requests to the barcode service URL
 */
export async function barcodeApi(): Promise<AxiosInstance> {
    const baseUrl = getBarcodeBaseUrl();

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
        apiBaseUrl: baseUrl,
        timeout: Config.timeOut,
        headers: () => {
            let headers: Record<string, string> = {};
            headers['Content-type'] = 'application/json';
            const currentUser = userStore.getState().current();
            if (currentUser?.bearer) {
                headers['Authorization'] = 'Bearer ' + currentUser.bearer;
            }
            return headers;
        },
        parseData: (data: any) => data,
        parseResult: ApiParse.parseResult,
        parseErrorResult: ApiParse.parseErrorResult,
    });
}
