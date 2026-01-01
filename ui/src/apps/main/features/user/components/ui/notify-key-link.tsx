import { Link } from "@tanstack/react-router"

interface NotifyKeyLinkProps {
    appId: string | number
    notifyMethod: string
    notifyKey: string | number
    className?: string
}

export function NotifyKeyLink({ appId, notifyMethod, notifyKey, className }: NotifyKeyLinkProps) {
    if (notifyMethod === 'sms_notify') {
        return (
            <Link
                to="/user/app/$appId/features-sms/list"
                params={{ appId: Number(appId) }}
                search={{ snid: String(notifyKey) }}
                className={className || "text-primary hover:underline"}
            >
                短信({notifyKey})
            </Link>
        )
    } else if (notifyMethod === 'sub_app_notify') {
        return (
            <Link
                to="/user/app/$appId/sub-app/list"
                params={{ appId: Number(appId) }}
                search={{ sub_app_id: Number(notifyKey) }}
                className={className || "text-primary hover:underline"}
            >
                子应用({notifyKey})
            </Link>
        )
    }

    return <span className={className}>{notifyKey || "-"}</span>
}
