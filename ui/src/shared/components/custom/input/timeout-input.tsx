import { AutocompleteInput } from "@shared/components/custom/input/autocomplete-input"
import { formatSeconds } from "@shared/lib/utils/format-utils"
import { cn } from "@shared/lib/utils/tools-utils"

interface TimeoutInputProps {
    value?: number
    onChange?: (value: number) => void
    placeholder?: string
    className?: string
    disabled?: boolean
    presetOptions?: number[]
}

const DEFAULT_PRESET_OPTIONS = [60, 3600, 14400, 43200, 604800, 0]

export function TimeoutInput({
    value = 0,
    onChange,
    placeholder = "请输入超时时间（秒）",
    className,
    disabled = false,
    presetOptions
}: TimeoutInputProps) {
    const options = presetOptions ?? DEFAULT_PRESET_OPTIONS

    const handleInputChange = (newValue: string) => {
        // Allow empty string or valid positive numbers only
        if (newValue === "" || /^\d+$/.test(newValue)) {
            const numValue = newValue === "" ? 0 : Number(newValue)
            onChange?.(numValue)
        }
    }

    const displayText = formatSeconds(value ?? 0)
    const autocompleteOptions = options.map(opt => ({
        value: String(opt),
        label: formatSeconds(opt)
    }))

    return (
        <div className={cn("flex items-center gap-2 w-full", className)}>
            <AutocompleteInput
                value={String(value ?? 0)}
                onChange={handleInputChange}
                placeholder={placeholder}
                disabled={disabled}
                options={autocompleteOptions}
                filterOnInput={false}
                className={cn("flex-1 min-w-0")}
            />
            <span className={cn("text-sm text-muted-foreground whitespace-nowrap flex-shrink-0")}>
                {displayText}
            </span>
        </div>
    )
}
