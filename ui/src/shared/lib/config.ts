import { z } from "zod";

// App host configuration for sub-services
const AppHostSchema = z.object({
    barcodeUrl: z.string().optional(),
});

const ConfigSchema = z.object({
    uiBasePath: z.string(),
    apiBaseUrl: z.string(),
    env: z.string(),
    timeOut: z.number().default(30000), // Default timeout of 30000ms (30秒)
    showDevtools: z.boolean().default(false), // 是否显示开发工具
    appHost: AppHostSchema.default({}),
});

type Config = z.infer<typeof ConfigSchema>;
export type AppHost = z.infer<typeof AppHostSchema>;

function getEnvConfig(): Config {
    const apiBaseUrl = process.env.API_BASE_URL;
    const uiBasePath = process.env.UI_BASE_PATH || '/';
    const env = process.env.ENV;
    const showDevtools = process.env.SHOW_DEVTOOLS === 'true';

    // App host configuration
    const appHost: z.infer<typeof AppHostSchema> = {
        barcodeUrl: process.env.BARCODE_BASE_URL,
    };

    if (apiBaseUrl && env) {
        return ConfigSchema.parse({ uiBasePath, apiBaseUrl, env, showDevtools, appHost });
    }
    throw new Error("Environment variables apiBaseUrl and ENV must be set");
}

export const Config = getEnvConfig();
