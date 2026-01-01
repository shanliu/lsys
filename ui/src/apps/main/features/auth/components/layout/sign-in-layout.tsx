import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@shared/components/ui/card';
import { cn } from '@shared/lib/utils';
import { isFromUserSwitch } from '@apps/main/lib/route-guards';
import { Link, Outlet, useLocation } from '@tanstack/react-router';
import { AppLogo } from '@apps/main/components/local/app-logo';

export function SignInLayout() {
  const location = useLocation();
  const hideSignUp = isFromUserSwitch(location);

  return (
    <div className='bg-primary-foreground container grid h-svh max-w-none items-center justify-center px-4 sm:px-0'>
      <div className='mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:w-[480px] sm:p-8'>
        <div className='mb-4 flex items-center justify-center'>
          <AppLogo alt="Logo" className='mr-2 h-5' linkToHome />
          <h1 className='text-xl font-medium'>登录</h1>
        </div>

        <Card className={cn('gap-4')}>
          <CardHeader className="pb-2">
            <CardTitle className={cn('text-lg tracking-tight flex items-center justify-between')}>
              <span>登录账号</span>
              {!hideSignUp && (
                <Link
                  to='/sign-up/sms'
                  className='hover:text-primary text-sm font-normal text-muted-foreground underline underline-offset-4'
                >
                  注册账号
                </Link>
              )}
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
