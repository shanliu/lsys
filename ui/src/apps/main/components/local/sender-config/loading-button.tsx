import { Button, buttonVariants } from "@shared/components/ui/button"
import { cn } from "@shared/lib/utils"
import { type VariantProps } from "class-variance-authority"
import { Loader2 } from "lucide-react"
import * as React from "react"

interface LoadingButtonProps
  extends React.ComponentProps<"button">,
  VariantProps<typeof buttonVariants> {
  loading?: boolean
  loadingText?: string
  asChild?: boolean
}

const LoadingButton = React.forwardRef<HTMLButtonElement, LoadingButtonProps>(
  ({ className, variant, size, asChild = false, loading = false, loadingText, children, disabled, ...props }, ref) => {
    return (
      <Button
        ref={ref}
        className={cn(className)}
        variant={variant}
        size={size}
        asChild={asChild}
        disabled={disabled || loading}
        {...props}
      >
        {loading && loadingText ? loadingText : children}
        {loading && <Loader2 className={cn("ml-auto h-4 w-4 animate-spin")} />}
      </Button>
    )
  }
)

LoadingButton.displayName = "LoadingButton"

export { LoadingButton }
