import {
  Activity,
  AppWindow,
  ClipboardList,
  Cog,
  FileSearch,
  FolderOpen,
  History,
  KeyRound,
  LogIn,
  Mail,
  MailCheck,
  MessageSquare,
  Route,
  Send,
  Settings,
  Shield,
  UserSearch,
  Users
} from 'lucide-react'
import type { MenuConfig } from '../types'

/**
 * 系统后台菜单
 */
export function getSystemMenu(): MenuConfig[] {
  return [
    {
      name: '应用管理',
      // 移除path和permission，组菜单权限由子项决定
      icon: AppWindow,
      children: [
        {
          name: '应用请求',
          path: '/admin/app/request',
          permission: 'admin:app:request',
          icon: ClipboardList
        },
        {
          name: '应用列表',
          path: '/admin/app/list',
          permission: 'admin:app:list',
          icon: FolderOpen
        }
      ]
    },
    {
      name: '邮件管理',
      // 移除path和permission，组菜单权限由子项决定
      icon: Mail,
      children: [
        {
          name: '系统邮件日志',
          path: '/admin/email/send-log',
          permission: 'admin:email:log',
          icon: FileSearch
        },
        {
          name: '系统邮件配置',
          path: '/admin/email/send-config/channel',
          permission: 'admin:email:config',
          icon: MailCheck
        },
        {
          name: '渠道信息配置',
          path: '/admin/email/adapter-config?type=smtp',
          permission: 'admin:email:adapter-config',
          icon: Route
        }
      ]
    },
    {
      name: '短信管理',
      // 移除path和permission，组菜单权限由子项决定
      icon: MessageSquare,
      children: [
        {
          name: '系统短信日志',
          path: '/admin/sms/send-log',
          permission: 'admin:sms:log',
          icon: History
        },
        {
          name: '系统短信配置',
          path: '/admin/sms/send-config/channel',
          permission: 'admin:sms:config',
          icon: Send
        },
        {
          name: '渠道信息配置',
          path: '/admin/sms/adapter-config?type=aliyun',
          permission: 'admin:sms:adapter-config',
          icon: Cog
        }
      ]
    },
    {
      name: '系统权限',
      // 移除path和permission，组菜单权限由子项决定
      icon: Shield,
      children: [
        {
          name: '授权日志',
          path: '/admin/rbac/audit-log',
          permission: 'admin:rbac:audit',
          icon: KeyRound
        },
        {
          name: '权限管理',
          path: '/admin/rbac/role',
          permission: 'admin:rbac:role',
          icon: Users
        }
      ]
    },
    {
      name: '用户管理',
      icon: Users,
      children: [
        {
          name: '搜索用户',
          path: '/admin/user/search',
          permission: 'admin:user:search',
          icon: UserSearch
        },
        {
          name: '操作日志',
          path: '/admin/user/change-log',
          permission: 'admin:user:change',
          icon: Activity
        },
        {
          name: '登陆日志',
          path: '/admin/user/login-log',
          permission: 'admin:user:login',
          icon: LogIn
        }
      ]
    },
    {
      name: '系统配置',
      icon: Settings,
      path: '/admin/config?type=site',
      permission: 'admin:config:site',
    }
  ]
}
