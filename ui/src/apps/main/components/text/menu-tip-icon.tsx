import { Button } from "@shared/components/ui/button"
import { cn } from "@shared/lib/utils"
import { AlertCircle } from "lucide-react"

export interface MenuTipIconProps {
    tipsMessage?: string
    className?: string
}

export function MenuTipIcon({ tipsMessage, className }: MenuTipIconProps) {
    if (!tipsMessage) {
        return null
    }

    return (
        <Button
            type="button"
            variant="ghost"
            size="icon"
            title={tipsMessage}
            className={cn("h-auto w-auto p-0.5 rounded-full cursor-help", className)}
            onClick={(e) => e.stopPropagation()}
        >
            <AlertCircle className="!size-3 shrink-0" aria-label={tipsMessage} />
        </Button>
    )
}
