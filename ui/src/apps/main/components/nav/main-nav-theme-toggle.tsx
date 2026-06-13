import { Button } from "@shared/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@shared/components/ui/dropdown-menu";
import { useTheme, type Theme } from "@shared/contexts/theme-context";
import { cn } from "@shared/lib/utils";
import { Monitor, Moon, Sun, type LucideIcon } from "lucide-react";
import { useMemo } from "react";

interface ThemeOption {
  value: Theme;
  label: string;
  icon: LucideIcon;
}

const THEME_OPTIONS: ThemeOption[] = [
  { value: 'light', label: '浅色', icon: Sun },
  { value: 'dark', label: '深色', icon: Moon },
  { value: 'system', label: '跟随系统', icon: Monitor },
] as const;

export function MainNavThemeToggle() {
  const { theme, setTheme } = useTheme();

  const currentThemeOption = useMemo(
    () => THEME_OPTIONS.find(option => option.value === theme) ?? THEME_OPTIONS[0],
    [theme]
  );

  const { icon: CurrentIcon, label: currentLabel } = currentThemeOption;

  const handleThemeChange = (newTheme: Theme) => {
    setTheme(newTheme);
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className={cn("h-9 w-9")}
          title={`当前主题: ${currentLabel}`}
          aria-label={`切换主题，当前：${currentLabel}`}
        >
          <CurrentIcon className={cn("h-4 w-4")} />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className={cn("w-36")}>
        {THEME_OPTIONS.map(({ value, label, icon: Icon }) => (
          <DropdownMenuItem
            key={value}
            onClick={() => handleThemeChange(value)}
            className={cn(
              "cursor-pointer",
              theme === value && "bg-accent"
            )}
          >
            <Icon className={cn("mr-2 h-4 w-4")} />
            <span>{label}</span>
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
