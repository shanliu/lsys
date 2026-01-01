import { QueryClient } from '@tanstack/react-query';
import { AxiosError } from 'axios';

export const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            retry: (failureCount: number, error: unknown) => {
                // 只对网络错误重试3次，其它错误不重试
                if (failureCount >= 3) return false;
                if (error && (error as AxiosError).name === 'AxiosError' && (error as AxiosError).code === 'ERR_NETWORK') return true;
                return false;
            },
            retryOnMount: false,
            refetchOnWindowFocus: false,
            staleTime: 5 * 60 * 1000, // 默认5分钟缓存
        },
        mutations: {
            onError: (error) => {
                if (error instanceof AxiosError) {
                    if (error.response?.status === 304) {
                        console.log('Content not modified!')
                    }
                }
            },
        },
    }
});
