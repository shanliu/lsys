import { create, StoreApi, UseBoundStore } from 'zustand';
import { persist, subscribeWithSelector } from 'zustand/middleware';
import { AuthUserActions, AuthUserData, AuthUserDataSchema, AuthUserItem } from './types';
import { authUserStatus } from './utils';

export type UserStore = UseBoundStore<StoreApi<AuthUserData & AuthUserActions>>;


// 创建持久化的 zustand store
export function createUserStore(storageKey: string): UserStore {
    return create(
        subscribeWithSelector(persist<AuthUserData & AuthUserActions>(
            (set, get) => ({
                useUserId: 0,
                userData: [],
                useUser: (data: AuthUserItem) => {
                    set((state: AuthUserData) => {
                        const userList = state?.userData || [];
                        const idx = userList.findIndex((item) => item.userId === data.userId);
                        let newUserData;
                        if (idx !== -1) {
                            // 存在则更新
                            newUserData = userList.map((item) =>
                                item.userId === data.userId ? { ...item, ...data } : item
                            );
                        } else {
                            // 不存在则新增
                            newUserData = [...userList, data];
                        }
                        return {
                            useUserId: data.userId,
                            userData: newUserData
                        };
                    });
                },
                delUser: (userId: number): AuthUserItem | null => {
                    let deletedUser: AuthUserItem | null = null;
                    set((state: AuthUserData) => {
                        if (userId === 0) {
                            deletedUser = null;
                            return state;
                        }
                        const userList = state.userData ?? [];
                        deletedUser = userList.find((item) => item.userId === userId) || null;
                        let userData = userList.filter((item) => item.userId !== userId);
                        let useUserId = state.useUserId;
                        if (state.useUserId === userId) {
                            // 被删除的是当前 useUserId
                            if (userData.length > 0) {
                                useUserId = userData[0].userId;
                            } else {
                                useUserId = 0;
                            }
                        }
                        return { userData, useUserId };
                    });
                    return deletedUser;
                },
                invalidatedUser: (userId: number, msg: string): AuthUserItem | null => {
                    let prevUser: AuthUserItem | null = null;
                    set((state: AuthUserData) => {
                        if (userId === 0) {
                            prevUser = null;
                            return state;
                        }
                        const userList = state.userData ?? [];
                        const idx = userList.findIndex((item) => item.userId === userId);
                        if (idx === -1) {
                            prevUser = null;
                            return state;
                        }
                        prevUser = userList[idx];
                        const updatedItem = { ...userList[idx], status: authUserStatus.Invalid(Math.ceil(Date.now() / 1000), msg) } as AuthUserItem;
                        const userData = userList.map((it, i) => (i === idx ? updatedItem : it));
                        return { userData, useUserId: state.useUserId };
                    });
                    return prevUser;
                },
                switchUser: (userId: number): [boolean, AuthUserItem | null] => {
                    let found = false;
                    let prevUser: AuthUserItem | null = null;
                    set((state: AuthUserData) => {
                        const userList = state.userData ?? [];
                        prevUser = userList.find((item) => item.userId === state.useUserId) || null;
                        // 检查目标用户是否存在
                        found = userList.some((item) => item.userId === userId);
                        return {
                            useUserId: userId,
                            userData: userList
                        };
                    });
                    return [found, prevUser || null];
                },
                current: (): AuthUserItem | null => {
                    const state = get() as AuthUserData;
                    if (!state) return null;
                    const userId = state.useUserId;
                    if (!userId) return null;
                    return (state.userData ?? []).find((item) => item.userId === userId) || null;
                },
                isLoggedIn: (): boolean => {
                    const state = get() as AuthUserData;
                    if (!state) return false;
                    const userId = state.useUserId;
                    if (!userId) return false;
                    return ((state.userData ?? []).find((item) => item.userId === userId) != null);
                },
                logout: (): AuthUserItem | null => {
                    let state = get() as AuthUserData;
                    const prevUser = (state.userData ?? []).find((item) => item.userId === state.useUserId) || null;
                    set((state: AuthUserData) => {
                        const userId = state.useUserId;
                        let userData = (state.userData ?? []).filter((item) => item.userId !== userId);
                        let useUserId = 0;
                        if (userData.length > 0) {
                            useUserId = userData[0].userId;
                        }
                        return { userData, useUserId };
                    });
                    return prevUser;
                },
                isExpired: (): boolean => {
                    const current = get().current?.();
                    if (!current) return true;
                    if (current.timeOut <= 0) return false;
                    const currentTime = Math.floor(Date.now() / 1000);
                    return currentTime > current.timeOut;
                },
            }),
            {
                name: storageKey,
                // 使用 onRehydrateStorage 来处理反序列化后的数据验证
                onRehydrateStorage: () => (state, error) => {
                    if (error) {
                        console.error('从本地存储恢复数据时出错:', error);
                        return;
                    }

                    if (state) {
                        Object.assign(state, validateStorageData(state));
                    }
                }
            }
        )
        ));
}



// 验证和清理从本地存储反序列化的数据
function validateStorageData(data: unknown): AuthUserData {
    try {
        // 使用 Zod schema 验证数据
        const validatedData = AuthUserDataSchema.parse(data);
        const validUserData = validatedData.userData.filter(user => {
            // 检查必要字段
            if (!user.bearer || user.userId <= 0) {
                console.warn(`过滤掉无效的用户数据: ${user.userId}`);
                return false;
            }
            return true;
        });
        // 检查当前使用的用户ID是否有效
        let useUserId = validatedData.useUserId;
        if (useUserId > 0 && !validUserData.find(user => user.userId === useUserId)) {
            console.warn(`当前使用的用户ID ${useUserId} 无效，重置为0`);
            useUserId = validUserData.length > 0 ? validUserData[0].userId : 0;
        }
        return {
            useUserId,
            userData: validUserData
        };
    } catch (error) {
        console.error('本地存储数据验证失败，使用默认值:', error);
        // 返回默认的空状态
        return {
            useUserId: 0,
            userData: []
        };
    }
}
