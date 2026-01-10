"use client"
import { format } from "date-fns"
import { zhCN } from "date-fns/locale"
import { Calendar as CalendarIcon } from "lucide-react"
import { useState } from "react"

import { Button } from "@shared/components/ui/button"
import { Calendar } from "@shared/components/ui/calendar"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@shared/components/ui/popover"
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@shared/components/ui/sheet"
import { cn } from "@shared/lib/utils"
import { useIsMobile } from "@shared/hooks/use-mobile"

interface DatePickerProps {
  value?: Date
  onChange?: (date: Date | undefined) => void
  placeholder?: string
  className?: string
  disabled?: boolean
  fromDate?: Date
  toDate?: Date
  fromYear?: number
  toYear?: number
}

export function DatePicker({
  value,
  onChange,
  placeholder = "请选择日期",
  className,
  disabled,
  fromDate,
  toDate,
  fromYear,
  toYear,
}: DatePickerProps) {
  const [open, setOpen] = useState(false)
  const isMobile = useIsMobile()

  const handleSelect = (date: Date | undefined) => {
    onChange?.(date)
    if (date) {
      setOpen(false)
    }
  }

  const triggerButton = (
    <Button
      variant="outline"
      className={cn(
        "h-10 w-full justify-start text-left font-normal",
        !value && "text-muted-foreground",
        className
      )}
      disabled={disabled}
    >
      <CalendarIcon className={cn("mr-2 h-4 w-4")} />
      {value ? (
        format(value, "yyyy年MM月dd日", { locale: zhCN })
      ) : (
        <span>{placeholder}</span>
      )}
    </Button>
  )

  // 移动端使用底部 Sheet
  if (isMobile) {
    return (
      <Sheet open={open} onOpenChange={setOpen}>
        <SheetTrigger asChild>
          {triggerButton}
        </SheetTrigger>
        <SheetContent 
          side="bottom" 
          className="px-4 pb-6 rounded-t-xl max-h-[70dvh] overflow-y-auto"
        >
          <SheetHeader className="text-left">
            <SheetTitle>{placeholder}</SheetTitle>
          </SheetHeader>
          <div className="flex justify-center pt-4 pb-4">
            <Calendar
              className={cn("w-full rounded-md [--cell-size:2.5rem]")}
              mode="single"
              selected={value}
              defaultMonth={value}
              onSelect={handleSelect}
              locale={zhCN}
              disabled={(date) => {
                if (fromDate && date < fromDate) return true
                if (toDate && date > toDate) return true
                return false
              }}
              captionLayout="dropdown"
              fromYear={fromYear}
              toYear={toYear}
              fromDate={fromDate}
              toDate={toDate}
              classNames={{
                nav: cn("absolute inset-x-0 top-2 flex w-full items-center justify-between gap-1 rdp-nav "),
              }}
            />
          </div>
        </SheetContent>
      </Sheet>
    )
  }

  // PC端使用 Popover（保持原有逻辑）
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        {triggerButton}
      </PopoverTrigger>
      <PopoverContent className={cn("w-auto min-w-[320px] p-0 rounded-md")} align="start">
        <Calendar
           className={cn("w-full rounded-md")}
          mode="single"
          selected={value}
          defaultMonth={value}
          onSelect={handleSelect}
          locale={zhCN}
          disabled={(date) => {
            if (fromDate && date < fromDate) return true
            if (toDate && date > toDate) return true
            return false
          }}
          captionLayout="dropdown"
          fromYear={fromYear}
          toYear={toYear}
          fromDate={fromDate}
          toDate={toDate}
          classNames={{
            nav: cn("absolute inset-x-0 top-2 flex w-full items-center justify-between gap-1 rdp-nav "),
          }}
        />
      </PopoverContent>
    </Popover>
  )
}
