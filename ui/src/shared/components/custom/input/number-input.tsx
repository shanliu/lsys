import { Minus, Plus } from "lucide-react"
import * as React from "react"

import { Button } from "@shared/components/ui/button"
import { Input } from "@shared/components/ui/input"
import { cn } from "@shared/lib/utils"

export interface NumberInputProps extends Omit<React.ComponentProps<"input">, "type" | "onChange" | "value"> {
    value?: number
    onChange?: (value: number) => void
    min?: number
    max?: number
    step?: number
    disabled?: boolean
}

const NumberInput = React.forwardRef<HTMLInputElement, NumberInputProps>(
    ({ className, value = 0, onChange, min, max, step = 1, disabled = false, ...props }, ref) => {
        const [internalValue, setInternalValue] = React.useState(value)

        React.useEffect(() => {
            setInternalValue(value)
        }, [value])

        const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
            const newValue = e.target.value === "" ? 0 : Number(e.target.value)
            updateValue(newValue)
        }

        const updateValue = (newValue: number) => {
            // 应用 min 和 max 约束
            let finalValue = newValue
            if (min !== undefined && finalValue < min) {
                finalValue = min
            }
            if (max !== undefined && finalValue > max) {
                finalValue = max
            }

            setInternalValue(finalValue)
            onChange?.(finalValue)
        }

        const increment = () => {
            const newValue = internalValue + step
            if (max === undefined || newValue <= max) {
                updateValue(newValue)
            }
        }

        const decrement = () => {
            const newValue = internalValue - step
            if (min === undefined || newValue >= min) {
                updateValue(newValue)
            }
        }

        const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
            if (e.key === "ArrowUp") {
                e.preventDefault()
                increment()
            } else if (e.key === "ArrowDown") {
                e.preventDefault()
                decrement()
            }
        }

        return (
            <div className={cn("flex items-center gap-2", className)}>
                <Input
                    ref={ref}
                    type="number"
                    value={internalValue}
                    onChange={handleInputChange}
                    onKeyDown={handleKeyDown}
                    disabled={disabled}
                    min={min}
                    max={max}
                    step={step}
                    className="flex-1"
                    {...props}
                />
                <Button
                    type="button"
                    variant="outline"
                    size="icon"
                    onClick={increment}
                    disabled={disabled || (max !== undefined && internalValue >= max)}
                    className="h-9 w-9 shrink-0"
                >
                    <Plus className="h-4 w-4" />
                </Button>
                <Button
                    type="button"
                    variant="outline"
                    size="icon"
                    onClick={decrement}
                    disabled={disabled || (min !== undefined && internalValue <= min)}
                    className="h-9 w-9 shrink-0"
                >
                    <Minus className="h-4 w-4" />
                </Button>
            </div>
        )
    }
)
NumberInput.displayName = "NumberInput"

export { NumberInput }

