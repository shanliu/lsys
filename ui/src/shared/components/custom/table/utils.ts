import { isValidElement } from "react";

/**
 * 检查一个 React 元素是否是 DataTableAction 组件
 */
export function isDataTableAction(element: any): boolean {
  if (!element || !isValidElement(element)) {
    return false;
  }
  const elementType = element.type as any;
  return (
    elementType?.displayName === "DataTableAction" ||
    elementType?.name === "DataTableAction"
  );
}
