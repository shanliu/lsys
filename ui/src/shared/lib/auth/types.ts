import { z } from "zod";


// 将 Zod schema 显式绑定到 TS 类型，保证二者一致（使用 discriminant 'kind'）
export const AuthUserStatusSchema = z.union([
    z.object({ kind: z.literal("Ok") }).strict(),
    z.object({ kind: z.literal("Invalid"), time: z.number(), msg: z.string().optional() }),
    z.object({ kind: z.literal("Expired"), expiredAt: z.number() }),
]);

export type AuthUserStatus = z.infer<typeof AuthUserStatusSchema>


export const AuthUserItemSchema = z.object({
    loginType: z.string(),
    userId: z.number(),
    userNikeName: z.string(),
    loginTime: z.number(),
    timeOut: z.number(),
    bearer: z.string(),
    loginData: z.any().optional(),
    appData: z.object({
        appId: z.number(),
        appName: z.string(),
        clientId: z.string(),
        changeTime: z.number(),
    }).nullable().optional().default(null),
    status: AuthUserStatusSchema,
});

export type AuthUserItem = z.infer<typeof AuthUserItemSchema>;

export const AuthUserDataSchema = z.object({
    useUserId: z.number().optional().default(0),
    userData: z.array(AuthUserItemSchema).default([]),
});

export type AuthUserData = z.infer<typeof AuthUserDataSchema>;

export type AuthUserActions = {
    //添加并使用指定用户完成登录
    useUser: (data: AuthUserItem) => void;
    //删除指定用户登录,并返回删除的登录信息
    delUser: (userId: number) => AuthUserItem | null;
    //切换到指定用户登录,并返回切换结果和上个登录信息
    switchUser: (userId: number) => [boolean, AuthUserItem | null];
    // 获取当前登录用户信息,如果没有登录返回 null
    current: () => AuthUserItem | null;
    // 退出当前登录用户,并返回退出的登录信息
    logout: () => AuthUserItem | null;
    // 将指定用户标记为失效
    invalidatedUser: (userId: number, msg: string) => AuthUserItem | null;
    // 判断当前用户是否已登录
    isLoggedIn: () => boolean;
    // 判断当前用户登录是否过期
    isExpired: () => boolean;
};
