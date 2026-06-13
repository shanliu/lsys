/**
 * @deprecated 请从 @apps/main/hooks 导入对应 hook：
 *   - 账号级导出：useUserExportAction from "@apps/main/hooks/use-user-export-action"
 *   - 应用级导出：useUserAppExportAction from "@apps/main/hooks/use-user-app-export-action"
 *
 * 此文件保留为向后兼容的适配层。
 */

export {
  useUserExportAction,
  type UseUserExportActionOptions,
  type UseExportActionResult,
  type UseExportActionResult as UseUserExportActionResult,
} from "@apps/main/hooks/use-user-export-action";

export {
  useUserAppExportAction,
  type UseUserAppExportActionOptions,
} from "@apps/main/hooks/use-user-app-export-action";
