/**
 * JWT Cross-Domain Sharing via postMessage
 * 
 * This module provides secure JWT sharing between main domain and sub-service domains.
 * 
 * Flow:
 * 1. Sub-service domain (e.g., barcode) sends JWT_REQUEST via postMessage to main domain
 * 2. Main domain validates the origin against appHost configuration
 * 3. If valid, main domain sends JWT_RESPONSE with the JWT token back
 * 4. Sub-service domain receives the token and can use it for authenticated requests
 */

import { Config } from '../config';
import { userStore } from './index';

// Message types for JWT sharing protocol
export const JWT_MESSAGE_TYPES = {
    REQUEST: 'LSYS_JWT_REQUEST',
    RESPONSE: 'LSYS_JWT_RESPONSE',
    LOGIN_REQUIRED: 'LSYS_LOGIN_REQUIRED',
} as const;

export interface JwtRequestMessage {
    type: typeof JWT_MESSAGE_TYPES.REQUEST;
    requestId: string;
}

export interface JwtResponseMessage {
    type: typeof JWT_MESSAGE_TYPES.RESPONSE;
    requestId: string;
    success: boolean;
    jwt?: string;
    userId?: number;
    error?: string;
}

export interface LoginRequiredMessage {
    type: typeof JWT_MESSAGE_TYPES.LOGIN_REQUIRED;
    reason: string;
    sourceOrigin: string;
}

type JwtMessage = JwtRequestMessage | JwtResponseMessage | LoginRequiredMessage;

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
 * Initialize JWT sharing listener on main domain
 * Should be called once when the main app starts
 */
export function initJwtSharingHost(): () => void {
    const allowedOrigins = getAllowedJwtOrigins();

    if (allowedOrigins.length === 0) {
        // No sub-services configured, no need to listen
        return () => { };
    }

    const handleMessage = (event: MessageEvent<JwtMessage>) => {
        // Validate origin
        if (!allowedOrigins.includes(event.origin)) {
            console.debug('[JWT Sharing] Rejected request from untrusted origin:', event.origin);
            return;
        }

        // Handle JWT request
        if (event.data?.type === JWT_MESSAGE_TYPES.REQUEST) {
            const request = event.data as JwtRequestMessage;
            console.debug('[JWT Sharing] Received JWT request from:', event.origin, 'requestId:', request.requestId);

            // Get current user's JWT
            const currentUser = userStore.getState().current();

            const response: JwtResponseMessage = {
                type: JWT_MESSAGE_TYPES.RESPONSE,
                requestId: request.requestId,
                success: !!currentUser?.bearer,
                jwt: currentUser?.bearer,
                userId: currentUser?.userId,
                error: currentUser ? undefined : 'User not logged in',
            };

            // Send response back to the requesting origin
            if (event.source && 'postMessage' in event.source) {
                (event.source as Window).postMessage(response, event.origin);
                console.debug('[JWT Sharing] Sent JWT response to:', event.origin, 'success:', response.success);
            }
            return;
        }

        // Handle login required notification from sub-service
        if (event.data?.type === JWT_MESSAGE_TYPES.LOGIN_REQUIRED) {
            const request = event.data as LoginRequiredMessage;
            console.debug('[JWT Sharing] Received login required from:', event.origin, 'reason:', request.reason);

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
    console.debug('[JWT Sharing] Host listener initialized for origins:', allowedOrigins);

    // Return cleanup function
    return () => {
        window.removeEventListener('message', handleMessage);
        console.debug('[JWT Sharing] Host listener removed');
    };
}

/**
 * Request JWT from main domain (used by sub-service domains)
 * 
 * @param mainOrigin - The origin of the main domain (e.g., 'https://main.example.com')
 * @param timeout - Timeout in milliseconds (default: 5000)
 * @returns Promise resolving to JWT token or null if not available
 */
export function requestJwtFromHost(mainOrigin: string, timeout = 5000): Promise<{ jwt: string; userId: number } | null> {
    return new Promise((resolve) => {
        const requestId = `jwt_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

        let timeoutId: ReturnType<typeof setTimeout>;
        let resolved = false;

        const handleResponse = (event: MessageEvent<JwtMessage>) => {
            // Validate origin
            if (event.origin !== mainOrigin) {
                return;
            }

            // Validate message type and requestId
            if (!event.data ||
                event.data.type !== JWT_MESSAGE_TYPES.RESPONSE ||
                (event.data as JwtResponseMessage).requestId !== requestId) {
                return;
            }

            if (resolved) return;
            resolved = true;

            clearTimeout(timeoutId);
            window.removeEventListener('message', handleResponse);

            const response = event.data as JwtResponseMessage;
            if (response.success && response.jwt && response.userId) {
                console.debug('[JWT Sharing] Received JWT from host');
                resolve({ jwt: response.jwt, userId: response.userId });
            } else {
                console.debug('[JWT Sharing] Host returned no JWT:', response.error);
                resolve(null);
            }
        };

        window.addEventListener('message', handleResponse);

        // Set timeout
        timeoutId = setTimeout(() => {
            if (resolved) return;
            resolved = true;
            window.removeEventListener('message', handleResponse);
            console.debug('[JWT Sharing] Request timed out');
            resolve(null);
        }, timeout);

        // Find the main window (parent or opener)
        const mainWindow = window.opener || window.parent;

        if (!mainWindow || mainWindow === window) {
            resolved = true;
            clearTimeout(timeoutId);
            window.removeEventListener('message', handleResponse);
            console.debug('[JWT Sharing] No parent/opener window found');
            resolve(null);
            return;
        }

        // Send request
        const request: JwtRequestMessage = {
            type: JWT_MESSAGE_TYPES.REQUEST,
            requestId,
        };

        mainWindow.postMessage(request, mainOrigin);
        console.debug('[JWT Sharing] Sent JWT request to:', mainOrigin);
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
 * Get allowed origins for JWT sharing based on appHost configuration
 * Used by main domain to validate postMessage origins
 */
export function getAllowedJwtOrigins(): string[] {
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
        console.debug('[JWT Sharing] No parent/opener window found, cannot notify main host');
        return;
    }

    const message: LoginRequiredMessage = {
        type: JWT_MESSAGE_TYPES.LOGIN_REQUIRED,
        reason,
        sourceOrigin: window.location.origin,
    };

    mainWindow.postMessage(message, mainOrigin);
    console.debug('[JWT Sharing] Sent login required to main host:', mainOrigin, 'reason:', reason);
}
