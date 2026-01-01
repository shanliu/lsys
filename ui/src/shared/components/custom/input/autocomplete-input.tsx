import { Input } from '@shared/components/ui/input';
import { cn } from '@shared/lib/utils';
import { Check, ChevronDown, Loader2 } from 'lucide-react';
import React, { useEffect, useRef, useState } from 'react';

interface AutocompleteOption {
  value: string;
  label: string;
}

interface AutocompleteInputProps {
  value?: string;
  onChange?: (value: string) => void;
  placeholder?: string;
  loading?: boolean;
  options?: string[] | AutocompleteOption[];
  disabled?: boolean;
  className?: string;
  id?: string;
  filterOnInput?: boolean;
}

export const AutocompleteInput: React.FC<AutocompleteInputProps> = ({
  value = '',
  onChange,
  placeholder,
  loading = false,
  options = [],
  disabled = false,
  className,
  id,
  filterOnInput = true,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [filteredOptions, setFilteredOptions] = useState<AutocompleteOption[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Normalize options to AutocompleteOption format
  const normalizedOptions: AutocompleteOption[] = React.useMemo(() =>
    options.map(opt =>
      typeof opt === 'string' ? { value: opt, label: opt } : opt
    ), [options]
  );

  // 过滤选项
  useEffect(() => {
    if (!filterOnInput || !value.trim()) {
      setFilteredOptions(normalizedOptions);
    } else {
      const filtered = normalizedOptions.filter(option =>
        option.label.toLowerCase().includes(value.toLowerCase()) ||
        option.value.toLowerCase().includes(value.toLowerCase())
      );
      setFilteredOptions(filtered);
    }
  }, [value, normalizedOptions, filterOnInput]);

  // 处理点击外部关闭下拉框
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node) &&
        !inputRef.current?.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    onChange?.(newValue);
    // 当启用输入过滤时，始终保持下拉框打开
    if (filterOnInput) {
      setIsOpen(true);
    } else if (!isOpen && newValue.trim()) {
      setIsOpen(true);
    }
  };

  const handleOptionClick = (option: AutocompleteOption) => {
    onChange?.(option.value);
    // 当启用输入过滤时，保持下拉框打开；否则关闭
    if (!filterOnInput) {
      setIsOpen(false);
    }
    inputRef.current?.focus();
  };

  const handleInputClick = () => {
    if (!loading && !disabled && (options.length > 0 || filteredOptions.length > 0)) {
      setIsOpen(!isOpen);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Escape') {
      setIsOpen(false);
    }
  };

  return (
    <div className={cn("relative", className)} style={{ overflow: 'visible' }}>
      <div className="relative">
        <Input
          ref={inputRef}
          id={id}
          value={value}
          onChange={handleInputChange}
          onClick={handleInputClick}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          disabled={disabled || (loading && !filterOnInput)}
          className={cn("pr-8")}
        />
        {loading ? (
          <Loader2 className={cn("absolute right-2 top-1/2 h-4 w-4 -translate-y-1/2 animate-spin text-muted-foreground")} />
        ) : options.length > 0 && !disabled ? (
          <ChevronDown
            className={cn(
              "absolute right-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground transition-transform cursor-pointer",
              isOpen && "rotate-180"
            )}
            onClick={handleInputClick}
          />
        ) : null}
      </div>

      {/* 加载状态指示器 */}
      {loading && !isOpen && (
        <div className="mt-2 flex items-center gap-2 text-sm text-muted-foreground">
          <Loader2 className={cn("h-3 w-3 animate-spin")} />
          <span>加载中...</span>
        </div>
      )}

      {/* 下拉选项列表 */}
      {isOpen && (filteredOptions.length > 0 || (!filterOnInput && normalizedOptions.length > 0) || (filterOnInput && loading)) && (
        <div
          ref={dropdownRef}
          className="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border border-border bg-background py-1 shadow-md"
          style={{
            position: 'absolute',
            top: '100%',
            left: '0',
            right: '0',
            maxHeight: '240px', // 限制最大高度
            overflowY: 'auto'
          }}
        >
          {!loading && (() => {
            const optionsToShow = filteredOptions.length > 0 ? filteredOptions : normalizedOptions;
            const unselectedOptions = optionsToShow.filter(opt => opt.value !== value);
            const selectedOption = optionsToShow.find(opt => opt.value === value);
            const sortedOptions = [...unselectedOptions, ...(selectedOption ? [selectedOption] : [])];

            return sortedOptions.map((option, index) => {
              const isSelected = !filterOnInput && option.value === value;
              return (
                <div
                  key={index}
                  onClick={() => handleOptionClick(option)}
                  className={cn(
                    "cursor-pointer px-3 py-2 text-sm hover:bg-muted text-foreground flex items-center gap-2",
                    isSelected && "bg-accent"
                  )}
                >
                  {!filterOnInput && (
                    <Check className={cn("h-4 w-4 flex-shrink-0", isSelected ? "opacity-100" : "opacity-0")} />
                  )}
                  <span className="flex-1">{option.label}</span>
                </div>
              );
            });
          })()}
          {loading && (
            <div className="flex items-center justify-center gap-2 py-4 text-sm text-muted-foreground">
              <Loader2 className={cn("h-4 w-4 animate-spin")} />
              <span>加载中...</span>
            </div>
          )}
        </div>
      )}

      {/* 无匹配选项提示 */}
      {isOpen && !loading && value.trim() && filteredOptions.length === 0 && options.length > 0 && (
        <div
          ref={dropdownRef}
          className="absolute z-50 mt-1 w-full rounded-md border border-border bg-background py-2 px-3 shadow-md"
          style={{
            position: 'absolute',
            top: '100%',
            left: '0',
            right: '0'
          }}
        >
          <div className="text-sm text-muted-foreground">无匹配选项</div>
        </div>
      )}
    </div>
  );
};
