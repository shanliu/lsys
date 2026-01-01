"use client"
import { format, setHours, setMinutes, setSeconds } from "date-fns"
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@shared/components/ui/select"
import { cn } from "@shared/lib/utils"

interface DateTimePickerProps {
  value?: Date
  onChange?: (date: Date | undefined) => void
  placeholder?: string
  className?: string
  disabled?: boolean
  fromDate?: Date
  toDate?: Date
  fromYear?: number
  toYear?: number
  minDateTime?: Date // 最小可选时间
}

export function DateTimePicker({
  value,
  onChange,
  placeholder = "请选择日期时间",
  className,
  disabled,
  fromDate,
  toDate,
  fromYear,
  toYear,
  minDateTime,
}: DateTimePickerProps) {
  const [open, setOpen] = useState(false)

  // 生成小时选项 (0-23)
  const hours = Array.from({ length: 24 }, (_, i) => i)
  // 生成分钟选项 (0-59)
  const minutes = Array.from({ length: 60 }, (_, i) => i)
  // 生成秒选项 (0-59)
  const seconds = Array.from({ length: 60 }, (_, i) => i)

  const handleDateSelect = (date: Date | undefined) => {
    if (date) {
      // 保留当前选择的时间，或使用默认时间
      const currentHour = value?.getHours() ?? new Date().getHours()
      const currentMinute = value?.getMinutes() ?? new Date().getMinutes()
      
      let newDate = setHours(date, currentHour)
      newDate = setMinutes(newDate, currentMinute)
      newDate = setSeconds(newDate, 0)
      
      // 如果设置了最小时间，确保新时间不早于最小时间
      if (minDateTime && newDate < minDateTime) {
        newDate = setHours(date, minDateTime.getHours())
        newDate = setMinutes(newDate, minDateTime.getMinutes() + 1)
        newDate = setSeconds(newDate, 0)
      }
      
      onChange?.(newDate)
    } else {
      onChange?.(undefined)
    }
  }

  const handleHourChange = (hour: string) => {
    if (value) {
      let newDate = setHours(value, parseInt(hour))
      
      // 如果设置了最小时间，确保新时间不早于最小时间
      if (minDateTime && newDate < minDateTime) {
        newDate = setMinutes(newDate, minDateTime.getMinutes() + 1)
      }
      
      onChange?.(newDate)
    } else {
      // 如果没有选择日期，先设置为今天
      const today = new Date()
      let newDate = setHours(today, parseInt(hour))
      newDate = setMinutes(newDate, 0)
      newDate = setSeconds(newDate, 0)
      onChange?.(newDate)
    }
  }

  const handleMinuteChange = (minute: string) => {
    if (value) {
      let newDate = setMinutes(value, parseInt(minute))
      newDate = setSeconds(newDate, 0)
      
      // 如果设置了最小时间，确保新时间不早于最小时间
      if (minDateTime && newDate < minDateTime) {
        return // 不允许设置早于最小时间的分钟
      }
      
      onChange?.(newDate)
    } else {
      // 如果没有选择日期，先设置为今天
      const today = new Date()
      let newDate = setHours(today, new Date().getHours())
      newDate = setMinutes(newDate, parseInt(minute))
      newDate = setSeconds(newDate, 0)
      onChange?.(newDate)
    }
  }

  const handleSecondChange = (second: string) => {
    if (value) {
      const newDate = setSeconds(value, parseInt(second))
      onChange?.(newDate)
    } else {
      // 如果没有选择日期，先设置为今天
      const today = new Date()
      let newDate = setHours(today, new Date().getHours())
      newDate = setMinutes(newDate, new Date().getMinutes())
      newDate = setSeconds(newDate, parseInt(second))
      onChange?.(newDate)
    }
  }

  // 检查某个小时是否可选（基于最小时间）
  const isHourDisabled = (hour: number): boolean => {
    if (!minDateTime || !value) return false
    
    // 如果是同一天，检查小时
    const isSameDay = value.toDateString() === minDateTime.toDateString()
    if (isSameDay && hour < minDateTime.getHours()) {
      return true
    }
    return false
  }

  // 检查某个分钟是否可选（基于最小时间）
  const isMinuteDisabled = (minute: number): boolean => {
    if (!minDateTime || !value) return false
    
    const isSameDay = value.toDateString() === minDateTime.toDateString()
    const isSameHour = value.getHours() === minDateTime.getHours()
    
    if (isSameDay && isSameHour && minute <= minDateTime.getMinutes()) {
      return true
    }
    return false
  }

  // 计算日历的 startMonth 和 endMonth
  const effectiveStartMonth = minDateTime && (!fromDate || minDateTime > fromDate) 
    ? new Date(minDateTime.getFullYear(), minDateTime.getMonth(), 1)
    : (fromDate ?? new Date())
  
  const effectiveEndMonth = toDate 
    ? new Date(toDate.getFullYear(), toDate.getMonth() + 1, 0)
    : new Date(new Date().getFullYear() + 10, 11, 31)

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
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
            format(value, "yyyy年MM月dd日 HH:mm:ss", { locale: zhCN })
          ) : (
            <span>{placeholder}</span>
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent className={cn("w-auto p-0")} align="start">
        <div className={cn("flex flex-col")}>
          <Calendar
            className={cn("rounded-md")}
            mode="single"
            selected={value}
            defaultMonth={value}
            onSelect={handleDateSelect}
            locale={zhCN}
            disabled={(date) => {
              const minDate = minDateTime 
                ? new Date(minDateTime.getFullYear(), minDateTime.getMonth(), minDateTime.getDate())
                : fromDate
              if (minDate && date < minDate) return true
              if (toDate && date > toDate) return true
              return false
            }}
            captionLayout="dropdown"
            startMonth={effectiveStartMonth}
            endMonth={effectiveEndMonth}
            classNames={{
              nav: cn("absolute inset-x-0 top-2 flex w-full items-center justify-between gap-1"),
              weekdays: cn("flex gap-1"),
              weekday: cn("text-muted-foreground w-8 select-none rounded-md text-[0.8rem] font-normal text-center"),
              week: cn("mt-2 flex w-full gap-1"),
              day: cn("group/day relative w-8 h-8 select-none p-0 text-center"),
            }}
          />
          <div className={cn("flex items-center justify-center gap-1 border-t px-3 py-2")}>
            <Select
              value={value?.getHours().toString()}
              onValueChange={handleHourChange}
            >
              <SelectTrigger className={cn("w-14 h-8 px-2")}>
                <SelectValue placeholder="时" />
              </SelectTrigger>
              <SelectContent className={cn("max-h-48")}>
                {hours.map((hour) => (
                  <SelectItem 
                    key={hour} 
                    value={hour.toString()}
                    disabled={isHourDisabled(hour)}
                  >
                    {hour.toString().padStart(2, '0')}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <span className={cn("text-muted-foreground")}>:</span>
            <Select
              value={value?.getMinutes().toString()}
              onValueChange={handleMinuteChange}
            >
              <SelectTrigger className={cn("w-14 h-8 px-2")}>
                <SelectValue placeholder="分" />
              </SelectTrigger>
              <SelectContent className={cn("max-h-48")}>
                {minutes.map((minute) => (
                  <SelectItem 
                    key={minute} 
                    value={minute.toString()}
                    disabled={isMinuteDisabled(minute)}
                  >
                    {minute.toString().padStart(2, '0')}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <span className={cn("text-muted-foreground")}>:</span>
            <Select
              value={value?.getSeconds().toString()}
              onValueChange={handleSecondChange}
            >
              <SelectTrigger className={cn("w-14 h-8 px-2")}>
                <SelectValue placeholder="秒" />
              </SelectTrigger>
              <SelectContent className={cn("max-h-48")}>
                {seconds.map((second) => (
                  <SelectItem 
                    key={second} 
                    value={second.toString()}
                  >
                    {second.toString().padStart(2, '0')}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              variant="outline"
              size="sm"
              className={cn("h-8 px-2 ml-1")}
              onClick={() => setOpen(false)}
            >
              确定
            </Button>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  )
}
