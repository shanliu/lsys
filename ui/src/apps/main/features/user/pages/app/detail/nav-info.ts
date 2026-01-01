import { SubNavigationMenuInfo } from '@apps/main/components/local/sub-navigation-menu'
import { Accessibility, Box, FileText, History, KeyRound, LogIn, Network, Puzzle, QrCode, Route, Ruler, ScanLine, Send, Users } from 'lucide-react'


/**
 * 邮件模块配置
 */
export const featureMailModuleConfig: SubNavigationMenuInfo = {
  title: '邮件发送',
  subtitle: '管理邮件发送记录、在线发送和配置',
  menuItems: [
    {
      name: '发送记录',
      icon: History,
      path: '/user/app/$appId/features-mail/list',
    },
    {
      name: '在线发送',
      icon: Send,
      path: '/user/app/$appId/features-mail/send',
    },

    {
      name: '发送渠道',
      icon: Route,
      path: '/user/app/$appId/features-mail/tpl-config',
    },
    {
      name: '模板管理',
      icon: FileText,
      path: '/user/app/$appId/features-mail/tpl',
    },
    {
      name: '发送规则',
      icon: Ruler,
      path: '/user/app/$appId/features-mail/config',
    },
  ],
}

/**
 * 短信模块配置
 */
export const featureSmsModuleConfig: SubNavigationMenuInfo = {
  title: '短信发送',
  subtitle: '管理短信发送记录、在线发送和配置',
  menuItems: [
    {
      name: '发送记录',
      icon: History,
      path: '/user/app/$appId/features-sms/list',
    },
    {
      name: '在线发送',
      icon: Send,
      path: '/user/app/$appId/features-sms/send',
    },
    {
      name: '发送渠道',
      icon: Route,
      path: '/user/app/$appId/features-sms/tpl-config',
    },
    {
      name: '发送规则',
      icon: Ruler,
      path: '/user/app/$appId/features-sms/config',
    },

  ],
}

/**
 * 二维码模块配置
 */
export const featureBarcodeModuleConfig: SubNavigationMenuInfo = {
  title: '二维码管理',
  subtitle: '创建和管理二维码',
  menuItems: [
    {
      name: '二维码创建列表',
      icon: QrCode,
      path: '/user/app/$appId/features-barcode/list-create',
    },
    {
      name: '二维码解析列表',
      icon: ScanLine,
      path: '/user/app/$appId/features-barcode/list-parse',
    },
  ],
}

/**
 * RBAC权限管理模块配置
 */
export const featureRbacModuleConfig: SubNavigationMenuInfo = {
  title: 'RBAC权限管理',
  subtitle: '管理角色和资源权限',
  menuItems: [
     {
      name: '授权日志',
      icon: Accessibility,
      path: '/user/app/$appId/features-rbac/audit',
    },
    {
      name: '角色管理',
      icon: Users,
      path: '/user/app/$appId/features-rbac/role',
    },
    {
      name: '资源管理',
      icon: Box,
      path: '/user/app/$appId/features-rbac/resource',
    },
  ],
}

/**
 * 服务权限模块扩展配置
 */
export interface ServiceModuleMenuInfo {
  /**
   * 有效菜单索引配置
   * 只有在此数组中的索引对应的菜单项才会显示
   * 如果未设置或为空数组，则显示所有菜单项
   */
  validMenuIndexes: number[]
}

/**
 * 服务权限模块配置
 */
export const serviceModuleConfig: SubNavigationMenuInfo & ServiceModuleMenuInfo = {
  title: '服务权限',
  subtitle: '管理应用服务和权限配置',
  validMenuIndexes: [0, 1],
  menuItems: [
    {
      name: '扩展功能权限',
      icon: Puzzle,
      path: '/user/app/$appId/service/feature',
    },
    {
      name: 'OAuth登陆权限',
      icon: KeyRound,
      path: '/user/app/$appId/service/oauth-client',
    },
    {
      name: '外部登录权限',
      icon: LogIn,
      path: '/user/app/$appId/service/exter-login',
    },
    {
      name: '子应用权限',
      icon: Network,
      path: '/user/app/$appId/service/sub-app',
    },
    {
      name: 'OAuth服务权限',
      icon: Users,
      path: '/user/app/$appId/service/sub-app-oauth-server',
    },
  ],
}

/**
 * 子应用模块配置
 */
export const subAppModuleConfig: SubNavigationMenuInfo = {
  title: '子应用管理',
  subtitle: '管理子应用列表和请求',
  menuItems: [],
}

/**
 * 首页模块配置
 */
export const indexModuleConfig: SubNavigationMenuInfo = {
  title: '应用详情',
  subtitle: '应用概况及统计',
  menuItems: [],
}

/**
 * 通知模块配置
 */
export const notifyModuleConfig: SubNavigationMenuInfo = {
  title: '回调通知',
  subtitle: '管理应用回调通知',
  menuItems: [],
}

/**
 * 请求提示模块配置
 */
export const requestPromptModuleConfig: SubNavigationMenuInfo = {
  title: '请求提示',
  menuItems: [],
}

/**
 * 请求模块配置
 */
export const requestModuleConfig: SubNavigationMenuInfo = {
  title: '请求记录',
  menuItems: [],
}

/**
 * 密钥模块配置
 */
export const secretModuleConfig: SubNavigationMenuInfo = {
  title: '应用密钥',
  menuItems: [],
}

/**
 * 设置模块配置
 */
export const settingModuleConfig: SubNavigationMenuInfo = {
  title: '应用设置',
  menuItems: [],
}

/**
 * 密钥管理直接访问URL
 * 在设置页面中添加 ?secret=true 参数即可直接打开密钥管理抽屉
 * 例如: /user/app/123/setting?secret=true
 */
