import { userStore } from '@shared/lib/auth';

// 新增 hook: useAuthUser，返回当前登录用户数据

export function useAuthData() {
	const userData= userStore((state) => state.current)();
	if (!userData) {
		throw new Error('User data is not available');
	}
	return userData;
}
