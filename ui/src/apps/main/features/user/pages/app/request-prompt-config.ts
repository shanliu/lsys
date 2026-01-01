import { Mail, MessageSquare, QrCode, Shield, AppWindow } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

/**
 * 服务类型配置
 */
export interface ServiceTypeConfig {
  /** 服务类型标识 */
  type: string
  /** 服务标题 */
  title: string
  /** 服务描述 */
  description: string
  /** 详细说明 */
  details: string[]
  /** 图标 */
  icon: LucideIcon
  /** 开通服务的路由路径 */
  serviceRoute: string
  /** 按钮文字 */
  buttonText: string
}

/**
 * 服务类型配置映射
 * 基于 getAppMenu 中的 noAuthPath type
 */
export const SERVICE_TYPE_CONFIGS: Record<string, ServiceTypeConfig> = {
  subapp: {
    type: 'subapp',
    title: '子应用管理功能',
    description: '您的应用尚未开通子应用管理功能',
    details: [
      '创建和管理子应用',
      '子应用申请审核',
      '子应用权限管理',
      '支持多层级应用架构'
    ],
    icon: AppWindow,
    serviceRoute: '/user/app/$appId/service/sub-app',
    buttonText: '开通子应用功能'
  },
  mail: {
    type: 'mail',
    title: '邮件服务',
    description: '您的应用尚未开通邮件服务功能',
    details: [
      '发送邮件通知',
      '邮件模板管理',
      '发送记录查询',
      '支持多种邮件服务商'
    ],
    icon: Mail,
    serviceRoute: '/user/app/$appId/service/feature',
    buttonText: '申请扩展功能'
  },
  sms: {
    type: 'sms',
    title: '短信服务',
    description: '您的应用尚未开通短信服务功能',
    details: [
      '发送短信验证码',
      '短信模板管理',
      '发送记录查询',
      '支持多家短信运营商'
    ],
    icon: MessageSquare,
    serviceRoute: '/user/app/$appId/service/feature',
    buttonText: '申请扩展功能'
  },
  barcode: {
    type: 'barcode',
    title: '条码服务',
    description: '您的应用尚未开通条码服务功能',
    details: [
      '生成二维码',
      '生成条形码',
      '支持多种编码格式',
      '可自定义样式和尺寸'
    ],
    icon: QrCode,
    serviceRoute: '/user/app/$appId/service/feature',
    buttonText: '申请扩展功能'
  },
  rbac: {
    type: 'rbac',
    title: '权限管理服务',
    description: '您的应用尚未开通权限管理功能',
    details: [
      '角色管理',
      '资源权限控制',
      '操作权限管理',
      '灵活的权限策略配置'
    ],
    icon: Shield,
    serviceRoute: '/user/app/$appId/service/feature',
    buttonText: '申请扩展功能'
  }
}

/**
 * 获取服务类型配置
 */
export function getServiceTypeConfig(type: string | undefined): ServiceTypeConfig | null {
  if (!type) return null
  return SERVICE_TYPE_CONFIGS[type] || null
}
