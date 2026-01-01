import z from "zod";



export const DictItemSchema = z.object({
    key: z.string(),
    val: z.string()
});

export type DictItemType = z.infer<typeof DictItemSchema>;

export const DictListSchema = z.array(DictItemSchema);
export type DictListType = z.infer<typeof DictListSchema>;

export class DictList extends Array<DictItemType> implements z.infer<typeof DictListSchema> {
    /**
     * 获取字典项的标签值
     * @param key - 字典项的键
     * @param fallback - 当键不存在时返回的默认值，默认返回键值本身
     * @returns 字典项的值，如果未找到则返回 fallback
     */
    getLabel(key: string, fallback: string = key): string {
        if (!this || !Array.isArray(this)) return fallback;
        const item = this.find(item => item.key === key);
        return item ? item.val : fallback;
    }

    /**
     * 将字典列表转换为选项数组，常用于下拉菜单等场景
     * @returns 包含 value 和 label 的选项数组
     */
    getOptions(): Array<{ value: string, label: string }> {
        if (!this || !Array.isArray(this)) return [];
        return this.map(item => ({ value: item.key, label: item.val }));
    }

    /**
     * 获取字典中所有的键
     * @returns 键的字符串数组
     */
    getKeys(): string[] {
        if (!this || !Array.isArray(this)) return [];
        return this.map(item => item.key);
    }

    /**
     * 检查字典中是否存在指定的键
     * @param key - 要检查的键
     * @returns 如果键存在返回 true，否则返回 false
     */
    hasKey(key: string): boolean {
        if (!this || !Array.isArray(this)) return false;
        return this.some(item => item.key === key);
    }

    /**
     * 查找并返回指定键的字典项
     * @param key - 要查找的键
     * @returns 找到的字典项，如果未找到则返回 undefined
     */
    findItem(key: string): DictItemType | undefined {
        if (!this || !Array.isArray(this)) return undefined;
        return this.find(item => item.key === key);
    }
}
