// ─────────────────────────────────────────────────────────
// runtime.std.encoding – 编码 / 解码 / ID 生成
//
// 注意: 运行在 QuickJS 引擎中，不是浏览器环境。
// 不使用 Annex B 函数 (escape / unescape)，因为 QuickJS
// 默认不启用 Annex B。所有 UTF-8 处理均手动实现。
//
//   runtime.std.encoding.btoa(str)             → Base64 编码 (Latin-1)
//   runtime.std.encoding.atob(str)             → Base64 解码 (Latin-1)
//   runtime.std.encoding.base64Encode(str)     → Base64 编码 (UTF-8 安全)
//   runtime.std.encoding.base64Decode(str)     → Base64 解码 (UTF-8 安全)
//   runtime.std.encoding.urlEncode(str)        → URL 编码
//   runtime.std.encoding.urlDecode(str)        → URL 解码
// ─────────────────────────────────────────────────────────
(function () {
    "use strict";

    var encoding = (runtime.std.encoding = {});

    // ── 内部工具 ─────────────────────────────────────────────

    var B64_CHARS =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    /** 构建 Base64 反查表 (char → index) */
    var B64_LOOKUP = {};
    (function () {
        for (var i = 0; i < B64_CHARS.length; i++) {
            B64_LOOKUP[B64_CHARS.charAt(i)] = i;
        }
    })();

    /**
     * 将 Unicode 字符串编码为 UTF-8 字节数组。
     * 纯 JS 实现，不依赖 escape / unescape。
     */
    function utf8Encode(str) {
        var bytes = [];
        for (var i = 0; i < str.length; i++) {
            var code = str.charCodeAt(i);
            // 处理代理对 (surrogate pair)
            if (code >= 0xd800 && code <= 0xdbff && i + 1 < str.length) {
                var lo = str.charCodeAt(i + 1);
                if (lo >= 0xdc00 && lo <= 0xdfff) {
                    code = ((code - 0xd800) << 10) + (lo - 0xdc00) + 0x10000;
                    i++;
                }
            }
            if (code <= 0x7f) {
                bytes.push(code);
            } else if (code <= 0x7ff) {
                bytes.push(0xc0 | (code >> 6));
                bytes.push(0x80 | (code & 0x3f));
            } else if (code <= 0xffff) {
                bytes.push(0xe0 | (code >> 12));
                bytes.push(0x80 | ((code >> 6) & 0x3f));
                bytes.push(0x80 | (code & 0x3f));
            } else {
                bytes.push(0xf0 | (code >> 18));
                bytes.push(0x80 | ((code >> 12) & 0x3f));
                bytes.push(0x80 | ((code >> 6) & 0x3f));
                bytes.push(0x80 | (code & 0x3f));
            }
        }
        return bytes;
    }

    /**
     * 将 UTF-8 字节数组解码为 Unicode 字符串。
     * 纯 JS 实现，不依赖 escape / unescape。
     */
    function utf8Decode(bytes) {
        var str = "", i = 0, len = bytes.length;
        while (i < len) {
            var b = bytes[i++];
            var code;
            if (b <= 0x7f) {
                code = b;
            } else if ((b & 0xe0) === 0xc0) {
                code = ((b & 0x1f) << 6) | (bytes[i++] & 0x3f);
            } else if ((b & 0xf0) === 0xe0) {
                code =
                    ((b & 0x0f) << 12) |
                    ((bytes[i++] & 0x3f) << 6) |
                    (bytes[i++] & 0x3f);
            } else if ((b & 0xf8) === 0xf0) {
                code =
                    ((b & 0x07) << 18) |
                    ((bytes[i++] & 0x3f) << 12) |
                    ((bytes[i++] & 0x3f) << 6) |
                    (bytes[i++] & 0x3f);
            } else {
                // 非法字节，跳过
                code = 0xfffd;
            }
            // 超过 BMP 需要代理对
            if (code > 0xffff) {
                code -= 0x10000;
                str += String.fromCharCode(0xd800 + (code >> 10));
                str += String.fromCharCode(0xdc00 + (code & 0x3ff));
            } else {
                str += String.fromCharCode(code);
            }
        }
        return str;
    }

    // ── Base64 (Latin-1) ─────────────────────────────────────

    /**
     * encoding.btoa(str) – Base64 encode (Web API naming)
     *
     * Encodes a Latin-1 / binary string to Base64.
     * Equivalent to `window.btoa()` in browsers.
     *
     * @param {string} str – Latin-1 input string
     * @returns {string} Base64 encoded string
     */
    encoding.btoa = function btoa(str) {
        var out = "", i = 0, len = str.length;
        var c1, c2, c3;
        while (i < len) {
            c1 = str.charCodeAt(i++) & 0xff;
            if (i >= len) {
                out += B64_CHARS.charAt(c1 >> 2);
                out += B64_CHARS.charAt((c1 & 0x3) << 4);
                out += "==";
                break;
            }
            c2 = str.charCodeAt(i++) & 0xff;
            if (i >= len) {
                out += B64_CHARS.charAt(c1 >> 2);
                out += B64_CHARS.charAt(((c1 & 0x3) << 4) | (c2 >> 4));
                out += B64_CHARS.charAt((c2 & 0xf) << 2);
                out += "=";
                break;
            }
            c3 = str.charCodeAt(i++) & 0xff;
            out += B64_CHARS.charAt(c1 >> 2);
            out += B64_CHARS.charAt(((c1 & 0x3) << 4) | (c2 >> 4));
            out += B64_CHARS.charAt(((c2 & 0xf) << 2) | (c3 >> 6));
            out += B64_CHARS.charAt(c3 & 0x3f);
        }
        return out;
    };

    /**
     * encoding.atob(str) – Base64 decode (Web API naming)
     *
     * Decodes a Base64 string back to a Latin-1 / binary string.
     * Equivalent to `window.atob()` in browsers.
     *
     * @param {string} str – Base64 encoded string
     * @returns {string} Decoded binary string
     */
    encoding.atob = function atob(str) {
        // strip whitespace and count padding
        str = str.replace(/[\s]/g, "");
        var pad = 0;
        if (str.charAt(str.length - 1) === "=") pad++;
        if (str.charAt(str.length - 2) === "=") pad++;
        str = str.replace(/=/g, "");

        var out = "", i = 0, len = str.length;
        while (i < len) {
            var a = B64_LOOKUP[str.charAt(i++)] || 0;
            var b = B64_LOOKUP[str.charAt(i++)] || 0;
            var c = B64_LOOKUP[str.charAt(i++)] || 0;
            var d = B64_LOOKUP[str.charAt(i++)] || 0;
            var triplet = (a << 18) | (b << 12) | (c << 6) | d;
            out += String.fromCharCode((triplet >> 16) & 0xff);
            if (i - 1 <= len - pad)
                out += String.fromCharCode((triplet >> 8) & 0xff);
            if (i <= len - pad) out += String.fromCharCode(triplet & 0xff);
        }
        return out;
    };

    // ── Base64 (UTF-8 safe) ──────────────────────────────────

    /**
     * encoding.base64Encode(str) – UTF-8 safe Base64 encode
     *
     * Handles full Unicode by encoding to UTF-8 bytes first,
     * then Base64-encoding those bytes.
     *
     * @param {string} str – Any Unicode string
     * @returns {string} Base64 encoded string
     */
    encoding.base64Encode = function base64Encode(str) {
        var bytes = utf8Encode(str);
        // bytes → Latin-1 string → btoa
        var latin1 = "";
        for (var i = 0; i < bytes.length; i++) {
            latin1 += String.fromCharCode(bytes[i]);
        }
        return encoding.btoa(latin1);
    };

    /**
     * encoding.base64Decode(str) – UTF-8 safe Base64 decode
     *
     * Inverse of base64Encode. Decodes Base64 then interprets
     * the bytes as UTF-8.
     *
     * @param {string} str – Base64 encoded string
     * @returns {string} Decoded Unicode string
     */
    encoding.base64Decode = function base64Decode(str) {
        var latin1 = encoding.atob(str);
        var bytes = [];
        for (var i = 0; i < latin1.length; i++) {
            bytes.push(latin1.charCodeAt(i) & 0xff);
        }
        return utf8Decode(bytes);
    };

    // ── URL encode / decode ──────────────────────────────────
    // encodeURIComponent / decodeURIComponent 属于 ECMAScript
    // 标准内置函数，QuickJS 完整支持。

    /**
     * encoding.urlEncode(str) – URL encode
     *
     * @param {string} str
     * @returns {string} Percent-encoded string
     */
    encoding.urlEncode = function urlEncode(str) {
        return encodeURIComponent(str);
    };

    /**
     * encoding.urlDecode(str) – URL decode
     *
     * @param {string} str – Percent-encoded string
     * @returns {string} Decoded string
     */
    encoding.urlDecode = function urlDecode(str) {
        return decodeURIComponent(str);
    };

})();
