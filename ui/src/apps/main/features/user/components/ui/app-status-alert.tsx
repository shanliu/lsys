import type { AppListItemType } from "@shared/apis/user/app"
import { Alert, AlertDescription, AlertTitle } from "@shared/components/ui/alert"
import { cn, formatTime, TIME_STYLE } from "@shared/lib/utils"
import { AlertCircle } from "lucide-react"

interface AppStatusAlertProps {
  appData: AppListItemType | null | undefined
}

export function AppStatusAlert({ appData }: AppStatusAlertProps) {
  if (!appData) {
    return null
  }

  // status=1: 应用待审核
  if (appData.status === 1) {
    return (
      <Alert className={cn("border-l-4")}>
        <AlertCircle className={cn("h-4 w-4")} />
        <AlertTitle className={cn("text-sm font-medium")}>应用待审核</AlertTitle>
        <AlertDescription className={cn("text-xs")}>
          您的应用正在审核中，{appData.change_time && (
            <span className={cn("text-muted-foreground")}>
              提交时间：{formatTime(appData.change_time, TIME_STYLE.ABSOLUTE_TEXT)}
            </span>
          )}
        </AlertDescription>
      </Alert>
    )
  }

  // status=3: 应用已禁用
  if (appData.status === 3) {
    return (
      <Alert className={cn("border-l-4")}>
        <AlertCircle className={cn("h-4 w-4")} />
        <AlertTitle className={cn("text-sm font-medium")}>应用已禁用</AlertTitle>
        <AlertDescription className={cn("text-xs")}>
          您的应用已被禁用。
          {appData.change_time && (
            <span className={cn("text-muted-foreground")}>
              禁用时间：{formatTime(appData.change_time, TIME_STYLE.ABSOLUTE_TEXT)}
            </span>
          )}
        </AlertDescription>
      </Alert>
    )
  }

  return null
}
