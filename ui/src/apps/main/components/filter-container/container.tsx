import {
  Drawer,
  DrawerContent,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from "@apps/main/components/local/drawer";
import { Button } from "@shared/components/ui/button";
import { useToast } from "@shared/contexts/toast-context";
import { useIsMobile } from "@shared/hooks/use-mobile";
import { cn } from "@shared/lib/utils";
import { ChevronDown, ChevronUp, Filter, MoreHorizontal } from "lucide-react";
import React from "react";
import {
  DefaultValues,
  FieldValues,
  FormProvider,
  Resolver,
  useForm,
  UseFormReturn,
} from "react-hook-form";

// 布局参数接口
export interface LayoutParams {
  isMobile: boolean;
}

// 扩展的 form 类型，包含自定义方法
export interface ExtendedFormReturn<
  TFieldValues extends FieldValues = FieldValues,
> extends UseFormReturn<TFieldValues> {
  // 预配置的提交处理函数
  handleFormSubmit: (e?: React.BaseSyntheticEvent) => Promise<void>;
  // 重置处理函数
  handleFormReset: () => void;
}

// 过滤器容器组件的 props 接口
export interface FilterContainerProps<
  TFieldValues extends FieldValues = FieldValues,
> {
  // 子组件渲染函数，接收布局参数和form方法
  children: (
    layoutParams: LayoutParams,
    form: ExtendedFormReturn<TFieldValues>,
  ) => React.ReactNode;
  // 自定义样式类名
  className?: string;
  // 表单提交处理函数
  onSubmit?: (data: TFieldValues) => void | Promise<void>;
  // 表单默认值
  defaultValues?: DefaultValues<TFieldValues>;
  // 表单重置时使用的初始值
  initValues?: DefaultValues<TFieldValues>;
  // 表单重置后的回调
  onReset?: () => void;
  // 表单解析器（支持 zod）
  resolver?: Resolver<TFieldValues>;
  // 总数显示组件（可选）
  countComponent?: React.ReactNode;
}

/**
 * 过滤器容器组件
 * 提供 react-hook-form 支持的过滤器容器，统一管理表单状态
 */
export function FilterContainer<
  TFieldValues extends FieldValues = FieldValues,
>({
  children,
  className,
  onSubmit,
  defaultValues,
  initValues,
  onReset,
  resolver,
  countComponent,
}: FilterContainerProps<TFieldValues>) {
  const isMobile = useIsMobile();
  const toast = useToast();

  // 移动端 Sheet 状态管理
  const [isSheetOpen, setIsSheetOpen] = React.useState(false);

  // PC端过滤器收缩状态管理
  const [isCollapsed, setIsCollapsed] = React.useState(false);

  // 初始化 react-hook-form
  const form = useForm<TFieldValues>({
    defaultValues,
    resolver,
    // 过滤器表单需要实时验证，在提交时触发 validation
    mode: "onSubmit",
  });

  // 当 defaultValues 变化时，重置表单（解决 URL 参数变化后表单不更新的问题）
  // React.useEffect(() => {
  //   form.reset(defaultValues);
  // }, [defaultValues, form]);

  // 处理表单提交
  const handleSubmit = React.useCallback(
    async (data: TFieldValues) => {
      // 检查是否有验证错误
      if (Object.keys(form.formState.errors).length > 0) {
        // 显示所有验证错误
        Object.entries(form.formState.errors).forEach(([field, error]) => {
          const message = typeof error?.message === 'string'
            ? error.message
            : `${field} 验证失败`;
          toast.error(message);
        });
        return;
      }
      // 验证成功才执行提交
      await onSubmit?.(data);
      // 移动端提交后关闭 Sheet
      if (isMobile) {
        setIsSheetOpen(false);
      }
    },
    [onSubmit, isMobile, toast, form.formState.errors],
  );

  // 处理表单重置
  const handleReset = React.useCallback(() => {
    const resetValues = initValues ?? (defaultValues
      ? Object.keys(defaultValues).reduce((acc, key) => {
        acc[key as keyof TFieldValues] = "" as any;
        return acc;
      }, {} as TFieldValues)
      : undefined);
    form.reset(resetValues);
    onReset?.();
  }, [form, initValues, defaultValues, onReset]);

  // 布局参数
  const layoutParams: LayoutParams = {
    isMobile,
  };

  // 扩展 form 对象，添加自定义的提交和重置方法
  const extendedForm = React.useMemo(
    (): ExtendedFormReturn<TFieldValues> => ({
      ...form,
      handleFormSubmit: form.handleSubmit(handleSubmit),
      handleFormReset: handleReset,
    }),
    [form, handleSubmit, handleReset],
  );

  // 监听表单值，计算已填写的筛选条件数量
  // 只统计 initValues 或 defaultValues 中定义的字段（筛选条件字段）
  const formValues = form.watch();
  const filterFields = initValues ?? defaultValues;
  const filledFilterCount = React.useMemo(() => {
    if (!filterFields) return 0;
    return Object.keys(filterFields).filter((key) => {
      const value = formValues[key];
      // 排除空值
      if (value === null || value === undefined) return false;
      // 字符串：排除空字符串和 "null"、"0"（通常表示未选择）
      if (typeof value === "string") {
        const trimmed = value.trim().toLowerCase();
        return trimmed !== "" && trimmed !== "null" && trimmed !== "0";
      }
      // 数组：只计算有元素的
      if (Array.isArray(value)) return value.length > 0;
      // 数字：0 通常表示未选择/全部，不计入
      if (typeof value === "number") return value !== 0;
      // 布尔值：只有 true 才计入
      if (typeof value === "boolean") return value === true;
      // 对象：只计算有属性的
      if (typeof value === "object") return Object.keys(value).length > 0;
      return false;
    }).length;
  }, [formValues, filterFields]);

  if (isMobile) {
    // 移动端：抽屉布局，支持左侧总数显示
    return (
      <FormProvider {...form}>
        <div className="flex items-center justify-between gap-2">
          {/* 左侧总数显示 */}
          <div className="flex-1 min-w-0">{countComponent}</div>

          {/* 右侧过滤按钮 */}
          <div className="flex-shrink-0">
            <Drawer open={isSheetOpen} onOpenChange={setIsSheetOpen}>
              <DrawerTrigger asChild>
                <Button variant="ghost" size="sm" className={cn("h-8 px-3")}>
                  <Filter className={cn("h-4 w-4 mr-2")} />
                  筛选
                  {filledFilterCount > 0 && (
                    <span className="ml-1.5 flex h-5 w-5 items-center justify-center rounded-full border border-current text-xs font-medium">
                      {filledFilterCount}
                    </span>
                  )}
                </Button>
              </DrawerTrigger>

              <DrawerContent
                className={cn("w-[95%] sm:max-w-md")}
                contentClassName="p-4"
              >
                <DrawerHeader className={cn("pb-4")}>
                  <DrawerTitle className={cn("flex items-center gap-2 text-left")}>
                    <Filter className={cn("h-4 w-4")} />
                    筛选条件
                  </DrawerTitle>
                </DrawerHeader>

                <div className="flex flex-col gap-4 overflow-y-auto [&>div]:!flex [&>div]:!flex-col [&>div]:!gap-4 [&>div>div]:!flex-1 [&>div>div]:!min-w-full [&>div>div]:!max-w-full">
                  {children(layoutParams, extendedForm)}
                </div>
              </DrawerContent>
            </Drawer>
          </div>
        </div>
      </FormProvider>
    );
  }

  // 桌面端：支持收缩展开
  return (
    <FormProvider {...form}>
      {isCollapsed ? (
        // 收缩状态：显示为长条
        <div
          className={cn(
            "relative flex items-center justify-center gap-2 px-4 py-2 bg-card border rounded-lg shadow-sm cursor-pointer hover:bg-accent/50 transition-colors",
            className,
          )}
          onClick={() => setIsCollapsed(false)}
        >
          <MoreHorizontal className={cn("h-4 w-4 text-muted-foreground")} />
          <span className="text-xs text-muted-foreground font-medium">
            当前筛选
          </span>
          {countComponent && (
            <div className="[&>*]:border-0 [&>*]:bg-transparent [&>*]:shadow-none">
              {countComponent}
            </div>
          )}
          <ChevronDown className={cn("h-4 w-4 text-muted-foreground ml-1")} />
        </div>
      ) : (
        // 展开状态：正常过滤器布局
        <div
          className={cn(
            "relative flex flex-col gap-3 px-4 pt-5 pb-5 bg-card border rounded-lg shadow-sm",
            className,
          )}
        >
          {/* 过滤器内容 */}
          <div className={cn("flex flex-wrap items-end gap-2 lg:gap-3")}>
            {children(layoutParams, extendedForm)}
          </div>

          {/* 桌面端总数显示和收缩按钮 - 绝对定位到右上角 */}
          <div className="absolute top-1.5 right-2 z-10 flex items-center gap-2">
            {countComponent}
            <Button
              variant="ghost"
              size="sm"
              className={cn("h-6 w-6 p-0 hover:bg-accent")}
              onClick={() => setIsCollapsed(true)}
            >
              <ChevronUp className={cn("h-3 w-3")} />
            </Button>
          </div>
        </div>
      )}
    </FormProvider>
  );
}
