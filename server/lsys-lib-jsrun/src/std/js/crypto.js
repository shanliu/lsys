// ─────────────────────────────────────────────────────────
// runtime.std.crypto – 常用哈希函数
//
//   runtime.std.crypto.md5(str)                → hex
//   runtime.std.crypto.sha1(str)               → hex
//   runtime.std.crypto.sha256(str)             → hex
//   runtime.std.crypto.hmacSha256(key, msg)    → hex
// ─────────────────────────────────────────────────────────
(function () {
    "use strict";

    var crypto = (runtime.std.crypto = {});

    // ── 通用工具 ─────────────────────────────────────────────

    /** 将字符串转为字节数组 (仅处理 Latin-1 范围) */
    function strToBytes(s) {
        var b = [];
        for (var i = 0; i < s.length; i++) b.push(s.charCodeAt(i) & 0xff);
        return b;
    }

    /** 字节数组转 hex 字符串 */
    function bytesToHex(bytes) {
        var hex = "";
        for (var i = 0; i < bytes.length; i++) {
            hex += ((bytes[i] >>> 4) & 0xf).toString(16);
            hex += (bytes[i] & 0xf).toString(16);
        }
        return hex;
    }

    /** 32-bit 安全加法 */
    function add32(a, b) {
        return ((a & 0xffff) + (b & 0xffff) + (((a >>> 16) + (b >>> 16)) << 16)) | 0;
    }

    // ── MD5 (RFC 1321) ──────────────────────────────────────

    crypto.md5 = (function () {
        function rotl(n, b) { return (n << b) | (n >>> (32 - b)); }
        function cmn(q, a, b, x, s, t) { return add32(rotl(add32(add32(a, q), add32(x, t)), s), b); }
        function ff(a, b, c, d, x, s, t) { return cmn((b & c) | (~b & d), a, b, x, s, t); }
        function gg(a, b, c, d, x, s, t) { return cmn((b & d) | (c & ~d), a, b, x, s, t); }
        function hh(a, b, c, d, x, s, t) { return cmn(b ^ c ^ d, a, b, x, s, t); }
        function ii(a, b, c, d, x, s, t) { return cmn(c ^ (b | ~d), a, b, x, s, t); }

        return function md5(str) {
            var bytes = strToBytes(str), len = bytes.length;
            // padding
            var words = [];
            for (var i = 0; i < len; i++) words[i >> 2] |= bytes[i] << ((i % 4) * 8);
            words[len >> 2] |= 0x80 << ((len % 4) * 8);
            var total = (((len + 8) >>> 6) + 1) * 16;
            while (words.length < total) words.push(0);
            words[total - 2] = (len * 8) | 0;

            var a = 0x67452301, b = 0xefcdab89, c = 0x98badcfe, d = 0x10325476;
            for (var i = 0; i < words.length; i += 16) {
                var aa = a, bb = b, cc = c, dd = d, x = words.slice(i, i + 16);
                a=ff(a,b,c,d,x[0],7,-680876936);   d=ff(d,a,b,c,x[1],12,-389564586);
                c=ff(c,d,a,b,x[2],17,606105819);    b=ff(b,c,d,a,x[3],22,-1044525330);
                a=ff(a,b,c,d,x[4],7,-176418897);    d=ff(d,a,b,c,x[5],12,1200080426);
                c=ff(c,d,a,b,x[6],17,-1473231341);  b=ff(b,c,d,a,x[7],22,-45705983);
                a=ff(a,b,c,d,x[8],7,1770035416);    d=ff(d,a,b,c,x[9],12,-1958414417);
                c=ff(c,d,a,b,x[10],17,-42063);      b=ff(b,c,d,a,x[11],22,-1990404162);
                a=ff(a,b,c,d,x[12],7,1804603682);   d=ff(d,a,b,c,x[13],12,-40341101);
                c=ff(c,d,a,b,x[14],17,-1502002290); b=ff(b,c,d,a,x[15],22,1236535329);
                a=gg(a,b,c,d,x[1],5,-165796510);    d=gg(d,a,b,c,x[6],9,-1069501632);
                c=gg(c,d,a,b,x[11],14,643717713);   b=gg(b,c,d,a,x[0],20,-373897302);
                a=gg(a,b,c,d,x[5],5,-701558691);    d=gg(d,a,b,c,x[10],9,38016083);
                c=gg(c,d,a,b,x[15],14,-660478335);  b=gg(b,c,d,a,x[4],20,-405537848);
                a=gg(a,b,c,d,x[9],5,568446438);     d=gg(d,a,b,c,x[14],9,-1019803690);
                c=gg(c,d,a,b,x[3],14,-187363961);   b=gg(b,c,d,a,x[8],20,1163531501);
                a=gg(a,b,c,d,x[13],5,-1444681467);  d=gg(d,a,b,c,x[2],9,-51403784);
                c=gg(c,d,a,b,x[7],14,1735328473);   b=gg(b,c,d,a,x[12],20,-1926607734);
                a=hh(a,b,c,d,x[5],4,-378558);       d=hh(d,a,b,c,x[8],11,-2022574463);
                c=hh(c,d,a,b,x[11],16,1839030562);  b=hh(b,c,d,a,x[14],23,-35309556);
                a=hh(a,b,c,d,x[1],4,-1530992060);   d=hh(d,a,b,c,x[4],11,1272893353);
                c=hh(c,d,a,b,x[7],16,-155497632);   b=hh(b,c,d,a,x[10],23,-1094730640);
                a=hh(a,b,c,d,x[13],4,681279174);    d=hh(d,a,b,c,x[0],11,-358537222);
                c=hh(c,d,a,b,x[3],16,-722521979);   b=hh(b,c,d,a,x[6],23,76029189);
                a=hh(a,b,c,d,x[9],4,-640364487);    d=hh(d,a,b,c,x[12],11,-421815835);
                c=hh(c,d,a,b,x[15],16,530742520);   b=hh(b,c,d,a,x[2],23,-995338651);
                a=ii(a,b,c,d,x[0],6,-198630844);    d=ii(d,a,b,c,x[7],10,1126891415);
                c=ii(c,d,a,b,x[14],15,-1416354905); b=ii(b,c,d,a,x[5],21,-57434055);
                a=ii(a,b,c,d,x[12],6,1700485571);   d=ii(d,a,b,c,x[3],10,-1894986606);
                c=ii(c,d,a,b,x[10],15,-1051523);    b=ii(b,c,d,a,x[1],21,-2054922799);
                a=ii(a,b,c,d,x[8],6,1873313359);    d=ii(d,a,b,c,x[15],10,-30611744);
                c=ii(c,d,a,b,x[6],15,-1560198380);  b=ii(b,c,d,a,x[13],21,1309151649);
                a=ii(a,b,c,d,x[4],6,-145523070);    d=ii(d,a,b,c,x[11],10,-1120210379);
                c=ii(c,d,a,b,x[2],15,718787259);    b=ii(b,c,d,a,x[9],21,-343485551);
                a = add32(a, aa); b = add32(b, bb); c = add32(c, cc); d = add32(d, dd);
            }
            // MD5 是 little-endian 输出
            var out = [];
            [a, b, c, d].forEach(function (w) {
                for (var i = 0; i < 4; i++) out.push((w >>> (i * 8)) & 0xff);
            });
            return bytesToHex(out);
        };
    })();

    // ── SHA-1 (FIPS 180-1) ──────────────────────────────────

    crypto.sha1 = function sha1(str) {
        var msg = strToBytes(str), len = msg.length;
        msg.push(0x80);
        while (msg.length % 64 !== 56) msg.push(0);
        var bits = len * 8;
        for (var i = 56; i >= 0; i -= 8) msg.push((bits / Math.pow(2, i)) & 0xff);

        var H0 = 0x67452301, H1 = 0xefcdab89, H2 = 0x98badcfe, H3 = 0x10325476, H4 = 0xc3d2e1f0;
        var W = new Array(80);

        for (var off = 0; off < msg.length; off += 64) {
            for (var t = 0; t < 16; t++)
                W[t] = (msg[off+t*4]<<24)|(msg[off+t*4+1]<<16)|(msg[off+t*4+2]<<8)|msg[off+t*4+3];
            for (var t = 16; t < 80; t++) {
                var x = W[t-3] ^ W[t-8] ^ W[t-14] ^ W[t-16];
                W[t] = (x << 1) | (x >>> 31);
            }
            var a=H0, b=H1, c=H2, d=H3, e=H4;
            for (var t = 0; t < 80; t++) {
                var f, k;
                if      (t < 20) { f = (b&c)|(~b&d);          k = 0x5a827999; }
                else if (t < 40) { f = b^c^d;                  k = 0x6ed9eba1; }
                else if (t < 60) { f = (b&c)|(b&d)|(c&d);     k = 0x8f1bbcdc; }
                else             { f = b^c^d;                  k = 0xca62c1d6; }
                var tmp = (((a<<5)|(a>>>27))+f+e+k+W[t])|0;
                e=d; d=c; c=(b<<30)|(b>>>2); b=a; a=tmp;
            }
            H0=add32(H0,a); H1=add32(H1,b); H2=add32(H2,c); H3=add32(H3,d); H4=add32(H4,e);
        }
        // big-endian 输出
        var out = [];
        [H0,H1,H2,H3,H4].forEach(function (w) {
            for (var i = 24; i >= 0; i -= 8) out.push((w >>> i) & 0xff);
        });
        return bytesToHex(out);
    };

    // ── SHA-256 (FIPS 180-4) ────────────────────────────────

    crypto.sha256 = (function () {
        var K = [
            0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
            0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
            0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
            0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
            0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
            0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
            0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
            0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2
        ];
        function rotr(n, x) { return (x >>> n) | (x << (32 - n)); }

        return function sha256(str) {
            var msg = strToBytes(str), len = msg.length;
            msg.push(0x80);
            while (msg.length % 64 !== 56) msg.push(0);
            var bits = len * 8;
            for (var i = 56; i >= 0; i -= 8) msg.push((bits / Math.pow(2, i)) & 0xff);

            var H = [0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19];
            var W = new Array(64);

            for (var off = 0; off < msg.length; off += 64) {
                for (var t = 0; t < 16; t++)
                    W[t] = (msg[off+t*4]<<24)|(msg[off+t*4+1]<<16)|(msg[off+t*4+2]<<8)|msg[off+t*4+3];
                for (var t = 16; t < 64; t++)
                    W[t] = add32(add32((rotr(17,W[t-2])^rotr(19,W[t-2])^(W[t-2]>>>10)), W[t-7]),
                                add32((rotr(7,W[t-15])^rotr(18,W[t-15])^(W[t-15]>>>3)), W[t-16]));

                var a=H[0],b=H[1],c=H[2],d=H[3],e=H[4],f=H[5],g=H[6],h=H[7];
                for (var t = 0; t < 64; t++) {
                    var T1 = add32(add32(add32(h, rotr(6,e)^rotr(11,e)^rotr(25,e)), (e&f)^(~e&g)), add32(K[t], W[t]));
                    var T2 = add32(rotr(2,a)^rotr(13,a)^rotr(22,a), (a&b)^(a&c)^(b&c));
                    h=g; g=f; f=e; e=add32(d,T1); d=c; c=b; b=a; a=add32(T1,T2);
                }
                H[0]=add32(H[0],a); H[1]=add32(H[1],b); H[2]=add32(H[2],c); H[3]=add32(H[3],d);
                H[4]=add32(H[4],e); H[5]=add32(H[5],f); H[6]=add32(H[6],g); H[7]=add32(H[7],h);
            }
            var out = [];
            for (var i = 0; i < 8; i++)
                for (var j = 24; j >= 0; j -= 8) out.push((H[i] >>> j) & 0xff);
            return bytesToHex(out);
        };
    })();

    // ── HMAC-SHA256 ─────────────────────────────────────────

    crypto.hmacSha256 = function hmacSha256(key, message) {
        var blockSize = 64, keyBytes = strToBytes(key);
        if (keyBytes.length > blockSize) {
            var h = crypto.sha256(key);
            keyBytes = [];
            for (var i = 0; i < h.length; i += 2) keyBytes.push(parseInt(h.substr(i, 2), 16));
        }
        while (keyBytes.length < blockSize) keyBytes.push(0);

        var oPad = "", iPad = "";
        for (var i = 0; i < blockSize; i++) {
            oPad += String.fromCharCode(keyBytes[i] ^ 0x5c);
            iPad += String.fromCharCode(keyBytes[i] ^ 0x36);
        }
        var inner = crypto.sha256(iPad + message);
        var innerRaw = "";
        for (var i = 0; i < inner.length; i += 2)
            innerRaw += String.fromCharCode(parseInt(inner.substr(i, 2), 16));
        return crypto.sha256(oPad + innerRaw);
    };


    /**
     * 参考 Web API `crypto.getRandomValues(typedArray)`。
     * 由于 QuickJS 无 TypedArray，返回普通 Array<number>，
     * 每个元素为 0–255 的随机整数。
     *
     * 支持两种调用：
     *   1) getRandomValues(length?)         -> number[]
     *   2) getRandomValues(arrayLikeObject) -> 原对象(就地填充)
     *
     * @param {number|Object} [target=16] – 字节数或带 length 的对象
     * @returns {number[]|Object} 随机字节数组或被填充后的原对象
     */
    crypto.getRandomValues = function getRandomValues(target) {
        var isNumber = typeof target === "number";
        var isArrayLike = target != null && typeof target === "object" && typeof target.length === "number";

        if (!isNumber && !isArrayLike && typeof target !== "undefined") {
            throw new TypeError("crypto.getRandomValues(target) expects a number or array-like object");
        }

        var length = 16;
        if (isNumber) {
            length = target;
        } else if (isArrayLike) {
            length = target.length;
        }

        if (typeof length !== "number" || !isFinite(length) || length < 0 || (length | 0) !== length) {
            throw new RangeError("crypto.getRandomValues length must be a non-negative integer");
        }

        // 和 Web Crypto 一致：长度上限为 65536 字节
        if (length > 65536) {
            throw new RangeError("crypto.getRandomValues length exceeds 65536 bytes");
        }

        var out = isArrayLike ? target : new Array(length);

        // 若宿主已有更强随机源，则优先使用
        if (
            typeof globalThis !== "undefined" &&
            globalThis.crypto &&
            typeof globalThis.crypto.getRandomValues === "function" &&
            globalThis.crypto !== crypto
        ) {
            // 尝试用普通数组进行填充（某些宿主可能仅接受 TypedArray）
            try {
                var temp = new Array(length);
                globalThis.crypto.getRandomValues(temp);
                for (var i = 0; i < length; i++) out[i] = temp[i] & 0xff;
                return out;
            } catch (e) {
                // 忽略并回退到 Math.random
            }
        }

        // 回退方案：非密码学安全随机（QuickJS 常见场景）
        for (var i = 0; i < length; i++) {
            out[i] = (Math.random() * 256) | 0;
        }

        return out;
    };
})();
