import {  MenuItem } from "@apps/main/types/ui-menu"

/**
 * 生成菜单项的唯一key
 * @param item 菜单项
 * @param prefix key的前缀
 * @param index 可选的索引号
 * @returns 唯一的key字符串
 */
export function generateMenuKey(item: MenuItem, prefix: string, index?: number): string {
    if (item.id) {
        return `${prefix}-${item.id}`
    }

    const indexSuffix = typeof index === 'number' ? `-${index}` : ''
    const namePart = item.name.toLowerCase().replace(/\s+/g, '-')
    return `${prefix}-${namePart}${indexSuffix}`
}

