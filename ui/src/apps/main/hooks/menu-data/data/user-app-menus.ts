import {
  Bell,
  ClipboardList,
  Cog,
  GitBranch,
  LayoutGrid,
  Mail,
  MessageSquare,
  Network,
  Puzzle,
  QrCode,
  Settings,
  Shield
} from 'lucide-react'
import { z } from 'zod'
import type { MenuConfig } from '../types'

// 定义菜单参数的zod schema - 使用 coerce 更简洁
const AppMenuParamsSchema = z.object({
  appId: z.coerce.number().int().positive("应用ID必须是正整数")
})

// 导出参数类型
export type AppMenuParams = z.infer<typeof AppMenuParamsSchema>

/**
 * 应用管理相关菜单 - 动态生成基于 appId
 */
export function getAppMenu(params: AppMenuParams): MenuConfig[] {
  // 使用zod直接验证并解构 - 简洁且类型安全
  const { appId } = AppMenuParamsSchema.parse(params)

  return [
    {
      name: '应用概览',
      path: '/user/app/$appId/',
      params: { appId },
      icon: LayoutGrid
    },
    {
      name: '应用管理',
      icon: Cog,
      children: [
        {
          name: '申请管理',
          path: '/user/app/$appId/request',
          params: { appId },
          icon: ClipboardList
        },
        {
          name: '功能管理',
          path: '/user/app/$appId/service/feature',
          params: { appId },
          icon: Puzzle
        },
        {
          name: '回调管理',
          path: '/user/app/$appId/notify',
          params: { appId },
          icon: Bell
        },
        {
          name: '应用设置',
          path: '/user/app/$appId/setting',
          params: { appId },
          icon: Settings
        }
      ]
    },
    {
      name: '子应用管理',
      icon: Network,
      permission: { name: 'app:subapp', data: { app_id: appId } },
      noAuthPath: "/user/app/$appId/request-prompt?type=subapp",
      params: { appId },
      showDisabled: true,
      // 移除path，让它纯粹作为分组菜单
      children: [
        {
          name: '子应用列表',
          path: '/user/app/$appId/sub-app/list',
          params: { appId },
          icon: GitBranch
        },
        {
          name: '子应用审核',
          path: '/user/app/$appId/sub-app/request',
          params: { appId },
          icon: ClipboardList
        }
      ]
    },
    {
      name: '扩展功能管理',
      icon: Puzzle,
      children: [
        {
          name: '邮件管理',
          path: '/user/app/$appId/features-mail/list',
          noAuthPath: "/user/app/$appId/request-prompt?type=mail",
          params: { appId },
          icon: Mail,
          permission: { name: 'app:mail', data: { app_id: appId } },
          showDisabled: true
        },
        {
          name: '短信管理',
          path: '/user/app/$appId/features-sms/list',
          noAuthPath: "/user/app/$appId/request-prompt?type=sms",
          params: { appId },
          icon: MessageSquare,
          permission: { name: 'app:sms', data: { app_id: appId } },
          showDisabled: true
        },
       
        {
          name: '权限管理',
          icon: Shield,
          permission: { name: 'app:rbac', data: { app_id: appId } },
          showDisabled: true,
          noAuthPath: "/user/app/$appId/request-prompt?type=rbac",
          params: { appId },
          path: '/user/app/$appId/features-rbac/audit',
        },
         {
          name: '条码管理',
          path: '/user/app/$appId/features-barcode/list-create',
          noAuthPath: "/user/app/$appId/request-prompt?type=barcode",
          params: { appId },
          icon: QrCode,
          permission: { name: 'app:barcode', data: { app_id: appId } },
          showDisabled: true
        },
      ]
    }
  ]
}
