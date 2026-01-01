import {
  AppWindow,
  History,
  KeyRound,
  LayoutGrid,
  Link,
  Lock,
  Mail,
  MapPin,
  PlusSquare,
  Smartphone,
  User,
  UserCog
} from 'lucide-react'
import type { MenuConfig } from '../types'

/**
 * 用户相关菜单
 */
export function getUserMenu(): MenuConfig[] {
  return [
    // 用户应用
    {
      name: '应用管理',
      icon: AppWindow,
      children: [
        // {
        //   name: '应用概况',
        //   path: '/user/app/dashboard',
        //   icon: AppWindow,
        // },
        {
          name: '应用列表',//change 待定
          path: '/user/app/list',
          icon: LayoutGrid,
        },
        {
          name: '应用申请',
          path: '/user/app/create',
          icon: PlusSquare,
        }
      ]
    },
    {
      name: '账号管理',//密码 登陆名 
      icon: Lock,
      permission: 'user:account',
      children: [
        {
          name: '登陆日志',
          path: '/user/account/login-log',
          icon: History,
        },
        {
          name: '绑定邮箱',
          path: '/user/account/email',
          icon: Mail,
        },
        {
          name: '绑定手机',
          path: '/user/account/mobile',
          icon: Smartphone,
        },
        {
          name: '绑定账号',
          path: '/user/account/external',
          icon: Link,
        },
        {
          name: '用户地址',
          path: '/user/account/address',
          icon: MapPin,
        },
        {
          name: '修改密码',
          path: '/user/account/change-password',
          icon: KeyRound,
        },
        {
          name: '登录账号',
          path: '/user/account/set-name',
          icon: User,
        },
        {
          name: '基本信息',
          path: '/user/account/set-info',
          icon: UserCog,
        },
      ]
    }
  ]
}
