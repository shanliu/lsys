// ─────────────────────────────────────────────────────────
// runtime.std.cache – Key-value cache (wraps core.Cache)
//
// 提供友好的缓存 API，底层使用 Rust moka 缓存引擎。
// 当 runtime 配置了 namespace 时，key 会自动加前缀隔离。
//
//   runtime.std.cache.get(key)                → 获取缓存值
//   runtime.std.cache.set(key, value, ttlMs?) → 设置缓存值
//   runtime.std.cache.has(key)                → 判断 key 是否存在
//   runtime.std.cache.remove(key)             → 删除缓存
//   runtime.std.cache.getJSON(key)            → 获取并 JSON.parse
//   runtime.std.cache.setJSON(key, obj, ttl?) → JSON.stringify 后存储
// ─────────────────────────────────────────────────────────
(function () {
    "use strict";

    var coreCache = runtime.core.Cache;

    var cache = (runtime.std.cache = {});

    /**
     * cache.get(key) – 获取缓存值
     *
     * @param {string} key
     * @returns {string|undefined} 缓存值，不存在返回 undefined
     */
    cache.get = function get(key) {
        var val = coreCache.get(key);
        return val === null ? undefined : val;
    };

    /**
     * cache.set(key, value, ttlMs?) – 设置缓存值
     *
     * @param {string} key
     * @param {string} value
     * @param {number} [ttlMs=0] – 过期时间(毫秒)，0 或不传使用引擎默认 TTL
     */
    cache.set = function set(key, value, ttlMs) {
        coreCache.set(key, String(value), ttlMs || 0);
    };

    /**
     * cache.has(key) – 判断 key 是否存在
     *
     * @param {string} key
     * @returns {boolean}
     */
    cache.has = function has(key) {
        return coreCache.has(key);
    };

    /**
     * cache.remove(key) – 删除缓存
     *
     * @param {string} key
     */
    cache.remove = function remove(key) {
        coreCache.remove(key);
    };

    /**
     * cache.getJSON(key) – 获取缓存并 JSON.parse
     *
     * @param {string} key
     * @returns {*} 解析后的对象，key 不存在返回 undefined
     */
    cache.getJSON = function getJSON(key) {
        var val = coreCache.get(key);
        if (val === null) return undefined;
        return JSON.parse(val);
    };

    /**
     * cache.setJSON(key, obj, ttlMs?) – JSON.stringify 后存入缓存
     *
     * @param {string} key
     * @param {*} obj – 任意可序列化的值
     * @param {number} [ttlMs=0]
     */
    cache.setJSON = function setJSON(key, obj, ttlMs) {
        coreCache.set(key, JSON.stringify(obj), ttlMs || 0);
    };
})();
