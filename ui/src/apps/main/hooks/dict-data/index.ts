/**
 * 字典数据系统入口文件
 * 
 * 统一导出字典相关的类型和配置，提供清晰的 API
 */

// 通用类型导出
export type { DictQueryContext } from './common';

// 导出参数解析函数
export { paramParse } from './common';

// 映射类型导出
export type { DictTypeMap } from './mappings';

// 运行时配置导出
export { dictQueryMap, dictQueryKeyMap } from './mappings';

// 字典定义导出（主要用于扩展）
export { dictDefinitions } from './dicts';
