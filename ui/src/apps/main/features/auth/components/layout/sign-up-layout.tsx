

import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@shared/components/ui/card'
import { cn } from '@shared/lib/utils'
import { Link, Outlet } from '@tanstack/react-router'
import { AppLogo } from '@apps/main/components/local/app-logo'

export function SignUpLayout() {
  return (
    <div className='bg-primary-foreground container grid h-svh max-w-none items-center justify-center px-4 sm:px-0'>
      <div className='mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:w-[480px] sm:p-8'>
        <div className='mb-4 flex items-center justify-center'>
          <AppLogo alt="Logo" className='mr-2 h-5' linkToHome />
          <h1 className='text-xl font-medium'>注册</h1>
        </div>

        <Card className={cn('gap-4')}>
          <CardHeader className="pb-2">
            <CardTitle className={cn('text-lg tracking-tight flex items-center justify-between')}>
              <span>创建账号</span>
              <Link
                to='/sign-in'
                className='hover:text-primary text-sm font-normal text-muted-foreground underline underline-offset-4'
              >
                已有账号？登录
              </Link>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <Outlet />
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
