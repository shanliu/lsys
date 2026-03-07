// ─────────────────────────────────────────────────────────
// runtime.std.params – Rust-side parameter / env bridge
//
//   runtime.std.getParams()                    → 获取全部任务参数对象
//   runtime.std.getEnv(name, defaultValue?)    → 获取系统环境变量
//
// 通过 core.message.postMessage 与 Rust MessageHandler 通信,
// 宿主侧按 message type 分发处理。
// ─────────────────────────────────────────────────────────
(function () {
    "use strict";

    var core = runtime.core;

    /**
     * runtime.std.getParams()
     *
     * Uses `core.message.postMessage(core.message.GET_PARAM, null)`
     * to ask the Rust-side MessageHandler for the entire parameters object.
     * Returns the whole params object directly.
     *
     * The message type identifier `GET_PARAM` is registered in Rust core
     * so both sides share the same constant.
     */
    runtime.std.getParams = function getParams() {
        return core.message.postMessage(
            core.message.GET_PARAM,
            null
        );
    };

    /**
     * runtime.std.getEnv(name, defaultValue?)
     *
     * Uses `core.message.postMessage(core.message.GET_ENV, { name })`
     * to ask the Rust-side MessageHandler for a system environment variable.
     * Expects the host to return `{ value, found }`.
     */
    runtime.std.getEnv = function getEnv(name, defaultValue) {
        var res = core.message.postMessage(
            core.message.GET_ENV,
            { name: name }
        );
        if (res && res.found) {
            return res.value;
        }
        if (arguments.length >= 2) {
            return defaultValue;
        }
        throw new Error("Environment variable '" + name + "' is not set");
    };
})();
