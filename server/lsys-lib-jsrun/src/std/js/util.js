// ─────────────────────────────────────────────────────────
// runtime.std utilities – sleep & fetch
//
// 完整 API 列表:
//   runtime.std.console       → Console API           (见 console.js)
//   runtime.std.crypto        → Web Crypto API        (见 crypto.js)
//   runtime.std.encoding      → 编码/解码             (见 encoding.js)
//   runtime.std.getParams()   → 获取全部任务参数      (见 params.js)
//   runtime.std.getEnv()      → 系统环境变量          (见 params.js)
//   runtime.std.cache         → 缓存                  (见 cache.js)
//   runtime.std.File          → 文件 I/O              (见 file.js)
//   runtime.std.fs            → Node.js 风格文件操作  (由 initGlobalsEnv 基于 File 构建)
//   runtime.std.sleep()       → 阻塞延迟
//
// initGlobalsEnv() 挂载的全局变量（Node.js 本地测试时在脚本顶部手动定义同名变量即可）:
//   params   ← runtime.std.getParams()  任务入参对象
//   getEnv   ← runtime.std.getEnv       环境变量函数
//   console  ← runtime.std.console
//   crypto   ← runtime.std.crypto
//   fetch    ← runtime.std.fetch
//   fs       ← Node.js 风格文件 API
//   btoa/atob← runtime.std.encoding
//   runtime.std.getRandomValues(len?) → 随机字节数组
//   runtime.std.randomHex(len?)       → 随机十六进制字符串
//   runtime.std.fetch()       → Fetch API
//   runtime.std.installWebGlobals() → 将 std API 挂到全局
// ─────────────────────────────────────────────────────────
(function () {
    "use strict";

    var core = runtime.core;

    // ── runtime.std.sleep – blocking delay ────────────────────
    // Wraps core.sleep to provide a setTimeout-style blocking sleep.
    // Unlike browser setTimeout, this is synchronous and blocks execution.

    /**
     * runtime.std.sleep(ms)
     *
     * Pauses execution for the specified number of milliseconds.
     * Bridges to `core.sleep(ms)` which uses `tokio::time::sleep`
     * on the Rust side via `block_in_place`.
     *
     * @param {number} ms – Duration in milliseconds (must be >= 0)
     */
    runtime.std.sleep = function sleep(ms) {
        if (typeof ms !== "number" || ms < 0) {
            throw new Error("sleep(ms) expects a non-negative number, got: " + ms);
        }
        core.sleep(ms);
    };

    // ── runtime.std.fetch – Fetch API style wrapper ──────────
    // Wraps core.fetch to provide a Response-like interface
    // consistent with the Web Fetch API.
    runtime.std.fetch = function fetch(url, options) {
        var req;
        if (options && typeof options === "object") {
            req = {};
            for (var k in options) {
                if (Object.prototype.hasOwnProperty.call(options, k)) {
                    req[k] = options[k];
                }
            }
            req.url = url;

            // ── cookie → Cookie header ──────────────────────────
            // Accept cookies as a string, object, or array and merge
            // them into the headers.Cookie field automatically.
            if (req.cookie != null) {
                if (!req.headers || typeof req.headers !== "object") {
                    req.headers = {};
                }
                var cookieStr;
                if (typeof req.cookie === "string") {
                    cookieStr = req.cookie;
                } else if (Array.isArray(req.cookie)) {
                    // ["a=1", "b=2"] → "a=1; b=2"
                    cookieStr = req.cookie.join("; ");
                } else if (typeof req.cookie === "object") {
                    // { a: "1", b: "2" } → "a=1; b=2"
                    var parts = [];
                    for (var ck in req.cookie) {
                        if (Object.prototype.hasOwnProperty.call(req.cookie, ck)) {
                            parts.push(ck + "=" + req.cookie[ck]);
                        }
                    }
                    cookieStr = parts.join("; ");
                }
                if (cookieStr) {
                    // Append to existing Cookie header if present
                    req.headers["Cookie"] = req.headers["Cookie"]
                        ? req.headers["Cookie"] + "; " + cookieStr
                        : cookieStr;
                }
                delete req.cookie;
            }
        } else {
            req = url;
        }

        var resp = core.fetch(req);
        var _body = resp.body;
        var _status = resp.status;
        return {
            status: _status,
            ok: _status >= 200 && _status < 300,
            statusText: _status >= 200 && _status < 300 ? "OK" : "Error",
            headers: {},
            url: url,
            text: function () {
                return _body;
            },
            json: function () {
                return JSON.parse(_body);
            },
        };
    };

    // ── runtime.std.initGlobalsEnv – browser/Node-like globals ──
    /**
     * 将 runtime.std 下常用 API 复制到 globalThis，模拟浏览器/Node 默认可访问对象。
     *
     * 仅在全局不存在时挂载，不覆盖已存在的全局对象。
     *
     * 挂载的全局变量：
     * - params  ← runtime.std.getParams()  任务入参对象
     *            【Node.js 本地测试】
     *            var params = JSON.parse(process.argv[2] || '{}');
     *            运行： node script.js '{"keyword":"foo"}'
     *            脚本主体代码无需修改，提交时删掉这一行即可。
     * - getEnv  ← runtime.std.getEnv      获取环境变量
     *            【Node.js 本地测试】function getEnv(k) { return process.env[k] || ""; }
     * - console ← runtime.std.console
     * - crypto  ← runtime.std.crypto
     * - fetch   ← runtime.std.fetch
     * - fs      ← Node.js 风格文件 API（基于 runtime.std.File 实现）
     *            【Node.js 本地测试】脚本顶部加：const fs = require('fs');
     * - Date    ← runtime.std.Date (若存在)
     * - btoa/atob← runtime.std.encoding.btoa/atob (若存在)
     *
     * 典型本地测试脚本头部（提交时删除此段）：
     * ──────────────────────────────────────────
     * // --- Node.js test shim (remove before submitting) ---
     * var params = JSON.parse(process.argv[2] || '{}');  // 命令行传入任务入参
     * // 运行： node script.js '{"keyword":"iPhone","page":1}'
     * function getEnv(k) { return process.env[k] || ""; }  // 模拟环境变量
     * const fs = require('fs');
     * // --- end shim ---
     * ──────────────────────────────────────────
     *
     * fs 方法列表（全部同步）：
     *   fs.readFileSync(name)                 → string
     *   fs.writeFileSync(name, data)          → void  (覆盖写入)
     *   fs.appendFileSync(name, data)         → void  (追加写入)
     *   fs.existsSync(name)                   → boolean
     *   fs.statSync(name)                     → { size, isFile(), isDirectory() }
     *   fs.unlinkSync(name)                   → void  (删除)
     *   fs.renameSync(old, new)               → void
     *   fs.copyFileSync(src, dest)            → void
     *   fs.readLines(name)                    → string[]  (非 Node.js 标准，sandbox 扩展)
     *   fs.writeLine(name, line)              → void  (追加一行)
     *   fs.writeCSVRow(name, fields, sep?)    → void  (追加一行 CSV)
     *   fs.writeCSVRows(name, rows, opts?)    → void  (追加多行 CSV)
     *   fs.writeTSVRow(name, fields)          → void  (追加一行 TSV)
     *   fs.writeTSVRows(name, rows, hdrs?)    → void  (追加多行 TSV)
     *   fs.writeJSON(name, obj, indent?)      → void  (覆盖写入 JSON)
     *   fs.writeJSONLines(name, arr)          → void  (追加 NDJSON)
     *   fs.localSync(name)                    → *     (同步到外部存储，仅 runtime 可用)
     *
     * @returns {object} 已挂载项摘要
     */
    runtime.std.initGlobalsEnv = function initGlobalsEnv() {
        var std = runtime.std || {};
        var g = typeof globalThis !== "undefined" ? globalThis : this;

        function canSet(name) {
            return typeof g[name] === "undefined";
        }

        function setIf(name, value) {
            if (typeof value === "undefined") return false;
            if (!canSet(name)) return false;
            g[name] = value;
            return true;
        }

        // ── 构造 fs 对象（依赖 std.File，由 file.js 提供）─────────────────
        var _fs;
        if (std.File) {
            var _F = std.File;

            // 内部工具：打开文件并定位到末尾（追加模式）
            function _openAppend(name) {
                var size = _F.getsize(name);
                var f = new _F(name);
                f.seek(size);
                return f;
            }

            _fs = {
                /** 读取文件全部内容 */
                readFileSync: function readFileSync(name) {
                    return _F.readAll(name);
                },
                /** 覆盖写入整个文件 */
                writeFileSync: function writeFileSync(name, data) {
                    _F.writeAll(name, String(data));
                },
                /** 追加写入到文件末尾 */
                appendFileSync: function appendFileSync(name, data) {
                    var f = _openAppend(name);
                    try { f.write(String(data)); }
                    finally { f.close(); }
                },
                /** 文件是否存在 */
                existsSync: function existsSync(name) {
                    return _F.exists(name);
                },
                /** 文件信息（size / isFile / isDirectory） */
                statSync: function statSync(name) {
                    return {
                        size: _F.getsize(name),
                        isFile: function () { return true; },
                        isDirectory: function () { return false; },
                    };
                },
                /** 删除文件 */
                unlinkSync: function unlinkSync(name) {
                    _F.remove(name);
                },
                /** 重命名文件 */
                renameSync: function renameSync(oldName, newName) {
                    var f = new _F(oldName);
                    try { f.rename(newName); }
                    finally { f.close(); }
                },
                /** 复制文件（覆盖目标） */
                copyFileSync: function copyFileSync(src, dest) {
                    _F.writeAll(dest, _F.readAll(src));
                },
                /** 按行读取，返回 string[]，自动处理 CRLF */
                readLines: function readLines(name) {
                    var lines = _F.readAll(name)
                        .replace(/\r\n/g, "\n")
                        .replace(/\r/g, "\n")
                        .split("\n");
                    if (lines.length > 0 && lines[lines.length - 1] === "") {
                        lines.pop();
                    }
                    return lines;
                },

                // ── 格式化写入（追加到文件末尾）──────────────────────────

                /**
                 * writeLine(name, line) – 追加一行（自动加 \n）
                 */
                writeLine: function writeLine(name, line) {
                    var f = _openAppend(name);
                    try { return f.writeLine(line); }
                    finally { f.close(); }
                },
                /**
                 * writeCSVRow(name, fields, sep?) – 追加一行 CSV
                 */
                writeCSVRow: function writeCSVRow(name, fields, sep) {
                    var f = _openAppend(name);
                    try { return f.writeCSVRow(fields, sep); }
                    finally { f.close(); }
                },
                /**
                 * writeCSVRows(name, rows, options?) – 追加多行 CSV
                 * options: { sep, headers }
                 */
                writeCSVRows: function writeCSVRows(name, rows, options) {
                    var f = _openAppend(name);
                    try { return f.writeCSVRows(rows, options); }
                    finally { f.close(); }
                },
                /**
                 * writeTSVRow(name, fields) – 追加一行 TSV
                 */
                writeTSVRow: function writeTSVRow(name, fields) {
                    var f = _openAppend(name);
                    try { return f.writeTSVRow(fields); }
                    finally { f.close(); }
                },
                /**
                 * writeTSVRows(name, rows, headers?) – 追加多行 TSV
                 */
                writeTSVRows: function writeTSVRows(name, rows, headers) {
                    var f = _openAppend(name);
                    try { return f.writeTSVRows(rows, headers); }
                    finally { f.close(); }
                },
                /**
                 * writeJSON(name, obj, indent?) – 覆盖写入 JSON（整文件替换）
                 */
                writeJSON: function writeJSON(name, obj, indent) {
                    var f = new _F(name);
                    try { f.seek(0); return f.writeJSON(obj, indent); }
                    finally { f.close(); }
                },
                /**
                 * writeJSONLines(name, arr) – 追加 JSON Lines (NDJSON)
                 */
                writeJSONLines: function writeJSONLines(name, arr) {
                    var f = _openAppend(name);
                    try { return f.writeJSONLines(arr); }
                    finally { f.close(); }
                },
                /**
                 * localSync(name) – 同步文件到外部存储（宿主 file_sync_handler）
                 * @returns {*} 宿主返回的 JSON 解析结果
                 */
                localSync: function localSync(name) {
                    var f = new _F(name);
                    try { return f.localSync(); }
                    finally { f.close(); }
                },
            };
        }

        var installed = {
            // 任务入参：runtime 下从 getParams() 取；Node.js 本地测试时 CLI 传入
            // var params = JSON.parse(process.argv[2] || '{}');
            // 运行: node script.js '{"key":"value"}'
            params: setIf("params", typeof std.getParams === "function" ? std.getParams() : undefined),
            // 环境变量：runtime 下委托给 std.getEnv；Node.js 本地测试时顶部加：
            // function getEnv(k) { return process.env[k] || ""; }
            getEnv: setIf("getEnv", typeof std.getEnv === "function" ? std.getEnv : undefined),
            console: setIf("console", std.console),
            crypto: setIf("crypto", std.crypto),
            fetch: setIf("fetch", std.fetch),
            fs: setIf("fs", _fs),
            Date: setIf("Date", std.Date),
            btoa: setIf("btoa", std.encoding && std.encoding.btoa),
            atob: setIf("atob", std.encoding && std.encoding.atob),
        };

        // 保证 crypto.getRandomValues 可直接用（可选覆盖）
        if (g.crypto && std.crypto && typeof std.crypto.getRandomValues === "function") {
            if (typeof g.crypto.getRandomValues !== "function") {
                g.crypto.getRandomValues = std.crypto.getRandomValues;
                installed.cryptoGetRandomValues = true;
            } else {
                installed.cryptoGetRandomValues = false;
            }
        } else {
            installed.cryptoGetRandomValues = false;
        }

        return installed;
    };
})();
