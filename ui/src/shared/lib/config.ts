import { z } from "zod";


const ConfigSchema = z.object({
    appBaseUrl: z.string(),
    apiBaseUrl: z.string(),
    env: z.string(),
    timeOut: z.number().default(30000), // Default timeout of 30000ms (30秒)
    showDevtools: z.boolean().default(false), // 是否显示开发工具
});

type Config = z.infer<typeof ConfigSchema>;

function getEnvConfig(): Config {
    const apiBaseUrl = process.env.API_BASE_URL;
    const appBaseUrl = process.env.APP_BASE_URL || '/';
    const env = process.env.ENV;
    const showDevtools = process.env.SHOW_DEVTOOLS === 'true';
    if (apiBaseUrl && env) {
        return ConfigSchema.parse({ appBaseUrl, apiBaseUrl, env, showDevtools });
    }
    throw new Error("Environment variables apiBaseUrl and ENV must be set");
}

export const Config = getEnvConfig();
