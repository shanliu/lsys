/**
 * Token Cross-Domain Sharing via postMessage
 *
 * This module provides secure auth token sharing between main domain and sub-service domains.
 *
 * Flow:
 * 1. Sub-service domain (e.g., barcode) sends TOKEN_REQUEST via postMessage to main domain
 * 2. Main domain validates the origin against appHost configuration
 * 3. If valid, main domain sends TOKEN_RESPONSE with the auth token back
 * 4. Sub-service domain receives the token and can use it for authenticated requests
 */

import { Config } from '../config';
import { userStore } from './index';

// Message types for token sharing protocol
export const TOKEN_MESSAGE_TYPES = {
    REQUEST: 'LSYS_TOKEN_REQUEST',
    RESPONSE: 'LSYS_TOKEN_RESPONSE',
    LOGIN_REQUIRED: 'LSYS_LOGIN_REQUIRED',
} as const;

export interface TokenRequestMessage {
    type: typeof TOKEN_MESSAGE_TYPES.REQUEST;
    requestId: string;
}

export interface TokenResponseMessage {
    type: typeof TOKEN_MESSAGE_TYPES.RESPONSE;
    requestId: string;
    success: boolean;
    token?: string;
    userId?: number;
    error?: string;
}

export interface LoginRequiredMessage {
    type: typeof TOKEN_MESSAGE_TYPES.LOGIN_REQUIRED;
    reason: string;
    sourceOrigin: string;
}

type TokenMessage = TokenRequestMessage | TokenResponseMessage | LoginRequiredMessage;

// Callbacks for handling login required from sub-services
let onLoginRequiredCallback: ((reason: string, sourceOrigin: string) => void) | null = null;

/**
 * Set callback for handling login required requests from sub-services
 * Main domain should set this to handle re-login
 */
export function setOnLoginRequired(callback: (reason: string, sourceOrigin: string) => void): void {
    onLoginRequiredCallback = callback;
}

/**
 * Initialize token sharing listener on main domain
 * Should be called once when the main app starts
 */
export function initTokenSharingHost(): () => void {
    const allowedOrigins = getAllowedTokenOrigins();

    if (allowedOrigins.length === 0) {
        // No sub-services configured, no need to listen
        return () => { };
    }

    const handleMessage = (event: MessageEvent<TokenMessage>) => {
        // Validate origin
        if (!allowedOrigins.includes(event.origin)) {
            console.debug('[Token Sharing] Rejected request from untrusted origin:', event.origin);
            return;
        }

        // Handle token request
        if (event.data?.type === TOKEN_MESSAGE_TYPES.REQUEST) {
            const request = event.data as TokenRequestMessage;
            console.debug('[Token Sharing] Received token request from:', event.origin, 'requestId:', request.requestId);

            // Get current user's token
            const currentUser = userStore.getState().current();

            const response: TokenResponseMessage = {
                type: TOKEN_MESSAGE_TYPES.RESPONSE,
                requestId: request.requestId,
                success: !!currentUser?.bearer,
                token: currentUser?.bearer,
                userId: currentUser?.userId,
                error: currentUser ? undefined : 'User not logged in',
            };

            // Send response back to the requesting origin
            if (event.source && 'postMessage' in event.source) {
                (event.source as Window).postMessage(response, event.origin);
                console.debug('[Token Sharing] Sent token response to:', event.origin, 'success:', response.success);
            }
            return;
        }

        // Handle login required notification from sub-service
        if (event.data?.type === TOKEN_MESSAGE_TYPES.LOGIN_REQUIRED) {
            const request = event.data as LoginRequiredMessage;
            console.debug('[Token Sharing] Received login required from:', event.origin, 'reason:', request.reason);

            if (onLoginRequiredCallback) {
                onLoginRequiredCallback(request.reason, request.sourceOrigin);
            } else {
                // Default: invalidate current user
                const state = userStore.getState();
                state.invalidatedUser(state.useUserId, request.reason);
            }
            return;
        }
    };

    window.addEventListener('message', handleMessage);
    console.debug('[Token Sharing] Host listener initialized for origins:', allowedOrigins);

    // Return cleanup function
    return () => {
        window.removeEventListener('message', handleMessage);
        console.debug('[Token Sharing] Host listener removed');
    };
}

/**
 * Request auth token from main domain (used by sub-service domains)
 *
 * @param mainOrigin - The origin of the main domain (e.g., 'https://main.example.com')
 * @param timeout - Timeout in milliseconds (default: 5000)
 * @returns Promise resolving to auth token or null if not available
 */
export function requestTokenFromHost(mainOrigin: string, timeout = 5000): Promise<{ token: string; userId: number } | null> {
    return new Promise((resolve) => {
        const requestId = `token_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

        let timeoutId: ReturnType<typeof setTimeout>;
        let resolved = false;

        const handleResponse = (event: MessageEvent<TokenMessage>) => {
            // Validate origin
            if (event.origin !== mainOrigin) {
                return;
            }

            // Validate message type and requestId
            if (!event.data ||
                event.data.type !== TOKEN_MESSAGE_TYPES.RESPONSE ||
                (event.data as TokenResponseMessage).requestId !== requestId) {
                return;
            }

            if (resolved) return;
            resolved = true;

            clearTimeout(timeoutId);
            window.removeEventListener('message', handleResponse);

            const response = event.data as TokenResponseMessage;
            if (response.success && response.token && response.userId) {
                console.debug('[Token Sharing] Received token from host');
                resolve({ token: response.token, userId: response.userId });
            } else {
                console.debug('[Token Sharing] Host returned no token:', response.error);
                resolve(null);
            }
        };

        window.addEventListener('message', handleResponse);

        // Set timeout
        timeoutId = setTimeout(() => {
            if (resolved) return;
            resolved = true;
            window.removeEventListener('message', handleResponse);
            console.debug('[Token Sharing] Request timed out');
            resolve(null);
        }, timeout);

        // Find the main window (parent or opener)
        const mainWindow = window.opener || window.parent;

        if (!mainWindow || mainWindow === window) {
            resolved = true;
            clearTimeout(timeoutId);
            window.removeEventListener('message', handleResponse);
            console.debug('[Token Sharing] No parent/opener window found');
            resolve(null);
            return;
        }

        // Send request
        const request: TokenRequestMessage = {
            type: TOKEN_MESSAGE_TYPES.REQUEST,
            requestId,
        };

        mainWindow.postMessage(request, mainOrigin);
        console.debug('[Token Sharing] Sent token request to:', mainOrigin);
    });
}

/**
 * Get main domain origin from API base URL
 */
export function getMainOrigin(): string {
    try {
        const url = new URL(Config.apiBaseUrl);
        return url.origin;
    } catch {
        return window.location.origin;
    }
}

/**
 * Get allowed origins for token sharing based on appHost configuration
 * Used by main domain to validate postMessage origins
 */
export function getAllowedTokenOrigins(): string[] {
    const origins: string[] = [];
    const { appHost } = Config;

    if (appHost.barcodeUrl) {
        try {
            const url = new URL(appHost.barcodeUrl);
            origins.push(url.origin);
        } catch {
            // Invalid URL, skip
        }
    }

    return origins;
}

/**
 * Notify main domain that login is required (used by sub-service domains)
 * 
 * @param mainOrigin - The origin of the main domain
 * @param reason - The reason for requiring login (e.g., '401 Unauthorized')
 */
export function requestMainHostLogin(mainOrigin: string, reason: string): void {
    // Find the main window (parent or opener)
    const mainWindow = window.opener || window.parent;

    if (!mainWindow || mainWindow === window) {
        console.debug('[Token Sharing] No parent/opener window found, cannot notify main host');
        return;
    }

    const message: LoginRequiredMessage = {
        type: TOKEN_MESSAGE_TYPES.LOGIN_REQUIRED,
        reason,
        sourceOrigin: window.location.origin,
    };

    mainWindow.postMessage(message, mainOrigin);
    console.debug('[Token Sharing] Sent login required to main host:', mainOrigin, 'reason:', reason);
}
