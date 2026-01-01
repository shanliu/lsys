import { SubNavigationMenuInfo } from '@apps/main/components/local/sub-navigation-menu'
import { FileText, Globe, KeyRound, Route, Ruler, Server, Shield, ShieldCheck } from 'lucide-react'

/**
 * 系统配置模块导航配置
 */
export const configModuleConfig: SubNavigationMenuInfo = {
  title: '系统配置管理',
  subtitle: '管理系统的各项配置信息',
  menuItems: [
    {
      name: '站点配置',
      icon: Globe,
      path: '/admin/config?type=site',
    },
    {
      name: 'OAuth配置',
      icon: KeyRound,
      path: '/admin/config?type=oauth',
    },
  ],
}

/**
 * 邮件适配器配置模块导航配置
 */
export const emailAdapterConfigModuleConfig: SubNavigationMenuInfo = {
  title: '邮件渠道配置',
  subtitle: '管理邮件发送适配器',
  menuItems: [
    {
      name: 'SMTP配置',
      icon: Server,
      path: '/admin/email/adapter-config?type=smtp',
    },
  ],
}

/**
 * 邮件发送配置模块导航配置
 */
export const emailSendConfigModuleConfig: SubNavigationMenuInfo = {
  title: '邮件发送配置',
  subtitle: '管理邮件模板、渠道和发送规则',
  menuItems: [
    {
      name: '发送渠道',
      icon: Route,
      path: '/admin/email/send-config/channel',
    },
    {
      name: '邮件模板',
      icon: FileText,
      path: '/admin/email/send-config/template',
    },
    {
      name: '发送规则',
      icon: Ruler,
      path: '/admin/email/send-config/rule',
    },
  ],
}

/**
 * 短信适配器配置模块导航配置
 */
export const smsAdapterConfigModuleConfig: SubNavigationMenuInfo = {
  title: '短信渠道配置',
  subtitle: '管理各云服务商的短信发送配置',
  menuItems: [
    {
      name: '阿里云短信',
      icon: Server,
      path: '/admin/sms/adapter-config?type=aliyun',
    },
    {
      name: '华为云短信',
      icon: Server,
      path: '/admin/sms/adapter-config?type=huawei',
    },
    {
      name: '腾讯云短信',
      icon: Server,
      path: '/admin/sms/adapter-config?type=tencent',
    },
    {
      name: '京东云短信',
      icon: Server,
      path: '/admin/sms/adapter-config?type=jd',
    },
    {
      name: '网易云短信',
      icon: Server,
      path: '/admin/sms/adapter-config?type=netease',
    },
    {
      name: '容联云短信',
      icon: Server,
      path: '/admin/sms/adapter-config?type=cloopen',
    },
  ],
}

/**
 * 短信发送配置模块导航配置
 */
export const smsSendConfigModuleConfig: SubNavigationMenuInfo = {
  title: '短信发送配置',
  subtitle: '管理短信模板配置和发送渠道',
  menuItems: [
    {
      name: '发送渠道',
      icon: Route,
      path: '/admin/sms/send-config/channel',
    },
    {
      name: '发送规则',
      icon: Ruler,
      path: '/admin/sms/send-config/rule',
    },
  ],
}

/**
 * RBAC权限管理模块导航配置
 */
export const rbacModuleConfig: SubNavigationMenuInfo = {
  title: 'RBAC权限管理',
  subtitle: '管理系统角色和资源权限',
  menuItems: [
    {
      name: '角色管理',
      icon: Shield,
      path: '/admin/rbac/role',
    },
    {
      name: '资源管理',
      icon: ShieldCheck,
      path: '/admin/rbac/resource',
    },
  ],
}
