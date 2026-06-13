import { useToast } from "@shared/contexts/toast-context";
import React from "react";
import {
  DefaultValues,
  FieldValues,
  Resolver,
  useForm,
} from "react-hook-form";
import type { UseFilterFormReturn } from "../components/filter-bar/context";

export interface UseFilterBarFormOptions<
  TFieldValues extends FieldValues = FieldValues,
> {
  /** 表单初始值（绑定到 URL 参数） */
  defaultValues?: DefaultValues<TFieldValues>;
  /**
   * 重置时使用的目标值。
   * 若不传，则将 defaultValues 中所有字段重置为空字符串。
   */
  initValues?: DefaultValues<TFieldValues>;
  /** Zod 等 schema 验证器 */
  resolver?: Resolver<TFieldValues>;
  /** 表单提交回调（验证通过后触发） */
  onSubmit?: (data: TFieldValues) => void | Promise<void>;
  /** 表单重置后的回调 */
  onReset?: () => void;
}

/**
 * 过滤器表单 hook
 *
 * 将 FilterContainer 中原本内置的表单逻辑提取出来，
 * 让页面组件能直接持有和操控表单状态，消除"不透明表单"问题。
 *
 * 返回的 form 对象可以：
 *   - 直接传给 <FilterContainer form={form}>
 *   - 在视图模式切换时调用 form.handleFormReset() 重置
 *   - 在任何地方调用 form.getValues() 读取当前筛选值
 *
 * @example
 * const filterForm = useFilterForm({
 *   defaultValues: { status: filterParam.status },
 *   onSubmit: (data) => navigate({ search: { ...data } }),
 *   onReset: () => navigate({ search: {} }),
 * });
 *
 * // 视图模式切换时可主动重置
 * const switchToOtherView = () => {
 *   filterForm.handleFormReset();
 *   setViewMode('other');
 * };
 *
 * return <FilterContainer form={filterForm}>...</FilterContainer>;
 */
export function useFilterBarForm<TFieldValues extends FieldValues = FieldValues>({
  defaultValues,
  initValues,
  resolver,
  onSubmit,
  onReset,
}: UseFilterBarFormOptions<TFieldValues>): UseFilterFormReturn<TFieldValues> {
  const toast = useToast();

  const form = useForm<TFieldValues>({
    defaultValues,
    resolver,
    mode: "onSubmit",
  });

  const handleSubmit = React.useCallback(
    async (data: TFieldValues) => {
      if (Object.keys(form.formState.errors).length > 0) {
        Object.entries(form.formState.errors).forEach(([field, error]) => {
          const message =
            typeof error?.message === "string"
              ? error.message
              : `${field} 验证失败`;
          toast.error(message);
        });
        return;
      }
      await onSubmit?.(data);
    },
    [onSubmit, toast, form.formState.errors],
  );

  const handleReset = React.useCallback(() => {
    const resetValues =
      initValues ??
      (defaultValues
        ? Object.keys(defaultValues).reduce((acc, key) => {
            acc[key as keyof TFieldValues] = "" as any;
            return acc;
          }, {} as TFieldValues)
        : undefined);
    form.reset(resetValues);
    onReset?.();
  }, [form, initValues, defaultValues, onReset]);

  const extendedForm = React.useMemo(
    () => ({
      ...form,
      handleFormSubmit: form.handleSubmit(handleSubmit),
      handleFormReset: handleReset,
    }),
    [form, handleSubmit, handleReset],
  );

  // 计算已填写的筛选条件数量（用于移动端徽标）
  const formValues = form.watch();
  const filterFields = initValues ?? defaultValues;
  const filledFilterCount = React.useMemo(() => {
    if (!filterFields) return 0;
    return Object.keys(filterFields).filter((key) => {
      const value = formValues[key];
      if (value === null || value === undefined) return false;
      if (typeof value === "string") {
        const trimmed = value.trim().toLowerCase();
        return trimmed !== "" && trimmed !== "null" && trimmed !== "0";
      }
      if (Array.isArray(value)) return value.length > 0;
      if (typeof value === "number") return value !== 0;
      if (typeof value === "boolean") return value === true;
      if (typeof value === "object") return Object.keys(value).length > 0;
      return false;
    }).length;
  }, [formValues, filterFields]);

  return { ...extendedForm, filledFilterCount };
}
