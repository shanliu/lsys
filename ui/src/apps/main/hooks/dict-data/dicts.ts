/**
 * 字典定义文件
 * 
 * 所有字典的单一数据源，每个字典只需定义一次：
 * - queryFn: 查询函数
 * - type: 响应类型
 */

import type { AppMapResType as adminAppMapRes } from '@shared/apis/admin/app';
import { appMapping as adminAppMapping } from '@shared/apis/admin/app';
import type { AdminExportMappingResType as adminExportMapRes } from '@shared/apis/admin/export';
import { adminExportTaskMapping as adminExportMappingFn } from '@shared/apis/admin/export';
import type { AdminFileMappingResType as adminFileMapRes } from '@shared/apis/admin/file';
import { adminFileMapping as adminFileMappingFn } from '@shared/apis/admin/file';
import type { RbacBaseMappingResType as adminRbacMapRes } from '@shared/apis/admin/rbac-base';
import { rbacBaseMapping as adminRbacBaseMapping } from '@shared/apis/admin/rbac-base';
import { systemSenderMailerMapping as adminSenderMailerMapping, SystemSenderMailerMapResType as adminSenderMailerMapRes } from '@shared/apis/admin/sender-mailer';
import { systemSenderSmsMapping as adminSenderSmsMapping, SystemSenderSmsMapResType as adminSenderSmsMapRes } from '@shared/apis/admin/sender-sms';
import { systemUserMapping as adminUserMapping, SystemUserMappingResType as adminUserMapRes } from '@shared/apis/admin/user';
import { appMapping as authLoginMapping, LoginMapResType as authLoginMapRes } from '@shared/apis/auth/login';
import { accountMapping as userAccountMapping, AccountMappingResType as userAccountMapRes } from '@shared/apis/user/account';
import { appMapping as userAppMapping, AppMapResType as userAppMapRes } from '@shared/apis/user/app';
import { userBarcodeMapping as userBarcodeMappingFn, UserBarcodeMapResType as userBarcodeMapRes } from '@shared/apis/user/barcode';
import { appRbacBaseMapping, AppRbacBaseMappingResType as userRbacMapRes } from '@shared/apis/user/rbac';
import { userSenderMailerMapping as userSenderMailerMappingFn, UserSenderMailerMappingResType as userSenderMailerMapRes } from '@shared/apis/user/sender-mailer';
import { userSenderSmsMapping as userSenderSmsMappingFn, UserSenderSmsMapResType as userSenderSmsMapRes } from '@shared/apis/user/sender-sms';
import { userFileMapping as userFileMappingFn, UserFileMappingResType as userFileMapRes } from '@shared/apis/user/file';
import { userExportTaskMapping as userExportMappingFn, UserExportMappingResType as userExportMapRes } from '@shared/apis/user/file';
import { userCollectorMapping as userCollectorMappingFn, UserCollectorMappingResType as userCollectorMapRes } from '@shared/apis/user/collector';
// user_rbac 目前与 admin_rbac 使用同一底层映射，如后续有独立 user 端 RBAC mapping 再调整
// NOTE: 某些 admin 与 user 共享同名字典，命名统一以所在模块前缀 + 功能名
import { userQueryKey } from '@apps/main/lib/auth-utils';
import type { DictQueryContext } from './common';

/**
 * 字典定义配置
 * 
 * 添加新字典时，只需在此处添加配置，所有相关类型和映射会自动生成
 * 
 * @example
 * ```typescript
 * // 添加新的字典类型
 * newDict: {
 *   queryKey: (params?: any) => userQueryKey('dict', 'newDict', params),
 *   queryFn: async (context: DictQueryContext, params?: any) => await newDictMapData(context),
 *   type: {} as NewDictMapRes,
 * },
 * ```
 */
export const dictDefinitions = {
  user_app: {
    queryKey: (params?: any) => userQueryKey('dict', 'user_app', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await userAppMapping({ signal: context.signal }),
    type: {} as userAppMapRes,
  },
  admin_app: {
    queryKey: (params?: any) => userQueryKey('dict', 'admin_app', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await adminAppMapping({ signal: context.signal }),
    type: {} as adminAppMapRes,
  },
  admin_rbac: {
    queryKey: (params?: any) => userQueryKey('dict', 'admin_rbac', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await adminRbacBaseMapping({ signal: context.signal }),
    type: {} as adminRbacMapRes,
  },
  // 认证相关字典 - 登录类型等
  auth_login: {
    queryKey: (params?: any) => userQueryKey('dict', 'auth_login', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await authLoginMapping({ signal: context.signal }),
    type: {} as authLoginMapRes,
  },
  // 用户端 - 账号相关字典
  user_account: {
    queryKey: (params?: any) => userQueryKey('dict', 'user_account', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await userAccountMapping({ signal: context.signal }),
    type: {} as userAccountMapRes,
  },
  // 用户端 - 条码相关字典
  user_barcode: {
    queryKey: (params?: any) => userQueryKey('dict', 'user_barcode', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await userBarcodeMappingFn({ signal: context.signal }),
    type: {} as userBarcodeMapRes,
  },
  // 用户端 - 短信发送服务字典
  user_sender_sms: {
    queryKey: (params?: any) => userQueryKey('dict', 'user_sender_sms', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await userSenderSmsMappingFn({ signal: context.signal }),
    type: {} as userSenderSmsMapRes,
  },
  // 用户端 - 邮件发送服务字典
  user_sender_mailer: {
    queryKey: (params?: any) => userQueryKey('dict', 'user_sender_mailer', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await userSenderMailerMappingFn({ signal: context.signal }),
    type: {} as userSenderMailerMapRes,
  },
  // 管理端 - 用户管理相关字典
  admin_user: {
    queryKey: (params?: any) => userQueryKey('dict', 'admin_user', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await adminUserMapping({ signal: context.signal }),
    type: {} as adminUserMapRes,
  },
  // 管理端 - 系统邮件发送服务字典
  admin_sender_mailer: {
    queryKey: (params?: any) => userQueryKey('dict', 'admin_sender_mailer', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await adminSenderMailerMapping({ signal: context.signal }),
    type: {} as adminSenderMailerMapRes,
  },
  // 管理端 - 系统短信发送服务字典
  admin_sender_sms: {
    queryKey: (params?: any) => userQueryKey('dict', 'admin_sender_sms', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await adminSenderSmsMapping({ signal: context.signal }),
    type: {} as adminSenderSmsMapRes,
  },
  // 用户端 - RBAC 权限管理字典
  app_rbac: {
    queryKey: (params?: any) => userQueryKey('dict', 'app_rbac', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await appRbacBaseMapping({}, { signal: context.signal }),
    type: {} as userRbacMapRes,
  },
  // 用户端 - 文件管理字典
  user_file: {
    queryKey: (params?: any) => userQueryKey('dict', 'user_file', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await userFileMappingFn({ signal: context.signal }),
    type: {} as userFileMapRes,
  },
  // 用户端 - 导出任务字典
  user_export: {
    queryKey: (params?: any) => userQueryKey('dict', 'user_export', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await userExportMappingFn({ signal: context.signal }),
    type: {} as userExportMapRes,
  },
  // 用户端 - 采集管理字典
  user_collector: {
    queryKey: (params?: any) => userQueryKey('dict', 'user_collector', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await userCollectorMappingFn({ signal: context.signal }),
    type: {} as userCollectorMapRes,
  },
  // 管理端 - 文件管理字典
  admin_file: {
    queryKey: (params?: any) => userQueryKey('dict', 'admin_file', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await adminFileMappingFn({ signal: context.signal }),
    type: {} as adminFileMapRes,
  },
  // 管理端 - 导出任务字典
  admin_export: {
    queryKey: (params?: any) => userQueryKey('dict', 'admin_export', params),
    queryFn: async (context: DictQueryContext, _params?: any) => await adminExportMappingFn({ signal: context.signal }),
    type: {} as adminExportMapRes,
  },
} as const;
