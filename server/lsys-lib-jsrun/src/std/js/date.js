// ─────────────────────────────────────────────────────────
// runtime.std.Date – 模拟 Web JS Date 对象
//
// 底层使用 core.localTime(ms?) 统一接口
// 由 Rust chrono 提供本地时区支持。
//
// 构造方式:
//   new runtime.std.Date()          → 当前时间
//   new runtime.std.Date(ms)        → 指定时间戳(毫秒)
//   new runtime.std.Date(dateStr)   → 解析日期字符串
//   new runtime.std.Date(y,m,d,h,min,s,ms) → 指定各部分
//
// 实例方法 (与 Web Date 一致):
//   getTime()           → UTC 毫秒时间戳
//   getFullYear()       → 本地年
//   getMonth()          → 本地月 (0-11, 与 Web 一致)
//   getDate()           → 本地日 (1-31)
//   getDay()            → 本地星期 (0=Sun)
//   getHours()          → 本地时 (0-23)
//   getMinutes()        → 本地分 (0-59)
//   getSeconds()        → 本地秒 (0-59)
//   getMilliseconds()   → 本地毫秒 (0-999)
//   getTimezoneOffset() → UTC 偏移(分钟), 与 Web 一致(西区正)
//   toISOString()       → ISO 8601 UTC 格式
//   toLocalString()     → 本地可读格式
//   toDateString()      → 本地日期部分
//   toTimeString()      → 本地时间部分
//   toString()          → 完整字符串
//   valueOf()           → 同 getTime()
//
// 静态方法:
//   Date.now()          → 当前 UTC 毫秒时间戳
//   Date.timezone()     → { offset, name }
//   Date.parse(str)     → 毫秒时间戳
// ─────────────────────────────────────────────────────────
(function () {
    "use strict";

    var core = runtime.core;

    // ── 内部工具 ─────────────────────────────────────────────

    /** 数字补零 */
    function pad(n, len) {
        var s = String(n < 0 ? -n : n);
        while (s.length < (len || 2)) s = "0" + s;
        return n < 0 ? "-" + s : s;
    }

    /** 简单日期字符串解析 → UTC ms, 支持:
     *   "2026-02-27"
     *   "2026-02-27T14:30:00"
     *   "2026-02-27T14:30:00Z"
     *   "2026-02-27T14:30:00+08:00"
     *   "2026/02/27 14:30:00"
     */
    function parseDateStr(str) {
        // 标准化分隔符
        str = str.replace(/\//g, "-").replace(" ", "T");

        var m = str.match(
            /^(\d{4})-(\d{1,2})-(\d{1,2})(?:T(\d{1,2}):(\d{1,2})(?::(\d{1,2})(?:\.(\d{1,3}))?)?)?(?:Z|([+-]\d{2}):?(\d{2}))?$/
        );
        if (!m) return NaN;

        var year  = parseInt(m[1], 10);
        var month = parseInt(m[2], 10) - 1; // 0-based
        var day   = parseInt(m[3], 10);
        var hours = parseInt(m[4] || "0", 10);
        var mins  = parseInt(m[5] || "0", 10);
        var secs  = parseInt(m[6] || "0", 10);
        var ms    = parseInt((m[7] || "0").substring(0, 3), 10);

        // 计算 UTC 毫秒: 简化实现使用儒略日算法
        var utcMs = dateToMs(year, month, day, hours, mins, secs, ms);

        // 应用时区偏移（如果有）
        if (m[8] !== undefined) {
            var tzH = parseInt(m[8], 10);
            var tzM = parseInt(m[9] || "0", 10);
            var tzOff = (tzH * 60 + (tzH < 0 ? -tzM : tzM)) * 60000;
            utcMs -= tzOff;
        } else if (str.indexOf("Z") === -1 && m[4] !== undefined) {
            // 无时区标识，视为本地时间，需加回本地偏移
            utcMs -= core.localTime().offset * 60000;
        }

        return utcMs;
    }

    /**
     * 将日期部分转为 UTC 毫秒时间戳
     * month 为 0-based (0=Jan)
     */
    function dateToMs(year, month, day, hours, mins, secs, ms) {
        // 使用公历到儒略日数的算法
        // 先调整 month 为 1-based
        var m = month + 1;
        var y = year;
        if (m <= 2) { y -= 1; m += 12; }
        var A = Math.floor(y / 100);
        var B = 2 - A + Math.floor(A / 4);
        var JD = Math.floor(365.25 * (y + 4716)) +
                 Math.floor(30.6001 * (m + 1)) +
                 day + B - 1524.5;
        // JD for Unix epoch (1970-01-01 00:00:00 UTC) = 2440587.5
        var daysSinceEpoch = JD - 2440587.5;
        return daysSinceEpoch * 86400000 +
               hours * 3600000 + mins * 60000 + secs * 1000 + (ms || 0);
    }

    // ── Date 构造函数 ────────────────────────────────────────

    /**
     * runtime.std.Date
     *
     * @constructor
     * @param {...*} args
     *   - ()              → 当前时间
     *   - (ms)            → 从 UTC 毫秒时间戳
     *   - (dateStr)       → 解析日期字符串
     *   - (y,m,d,h,min,s,ms) → 各部分 (本地时间, month 0-based)
     */
    function StdDate() {
        if (!(this instanceof StdDate)) {
            // 不用 new 调用时返回字符串（与 Web Date 行为一致）
            return new StdDate().toString();
        }

        var argc = arguments.length;

        if (argc === 0) {
            this._ms = core.localTime().now;
        } else if (argc === 1) {
            var arg = arguments[0];
            if (typeof arg === "number") {
                this._ms = arg;
            } else if (typeof arg === "string") {
                this._ms = parseDateStr(arg);
            } else {
                this._ms = NaN;
            }
        } else {
            // new Date(year, month [, day, hours, minutes, seconds, ms])
            // month 是 0-based, 视为本地时间
            var y = arguments[0];
            var m = arguments[1];
            var d = argc > 2 ? arguments[2] : 1;
            var h = argc > 3 ? arguments[3] : 0;
            var mi = argc > 4 ? arguments[4] : 0;
            var s = argc > 5 ? arguments[5] : 0;
            var milli = argc > 6 ? arguments[6] : 0;

            // 构建为本地时间再转 UTC
            var utcMs = dateToMs(y, m, d, h, mi, s, milli);
            // 减去本地偏移得到 UTC
            this._ms = utcMs - core.localTime().offset * 60000;
        }

        // 缓存本地时间部分 (惰性)
        this._parts = null;
    }

    /** 获取本地时间部分 (按需从 Rust 计算) */
    StdDate.prototype._local = function () {
        if (!this._parts) {
            this._parts = core.localTime(this._ms);
        }
        return this._parts;
    };

    // ── Getter 方法 ──────────────────────────────────────────

    StdDate.prototype.getTime = function () { return this._ms; };
    StdDate.prototype.valueOf = function () { return this._ms; };

    StdDate.prototype.getFullYear = function () { return this._local().year; };
    StdDate.prototype.getMonth = function () { return this._local().month - 1; }; // 0-based
    StdDate.prototype.getDate = function () { return this._local().day; };
    StdDate.prototype.getDay = function () { return this._local().weekday; };
    StdDate.prototype.getHours = function () { return this._local().hours; };
    StdDate.prototype.getMinutes = function () { return this._local().minutes; };
    StdDate.prototype.getSeconds = function () { return this._local().seconds; };
    StdDate.prototype.getMilliseconds = function () { return this._local().ms; };

    /**
     * getTimezoneOffset() – 返回 UTC 偏移(分钟)
     *
     * 与 Web Date 一致: 东区返回负数。
     * 例如 UTC+8 返回 -480。
     */
    StdDate.prototype.getTimezoneOffset = function () {
        return -(this._local().offset);
    };

    // ── 格式化方法 ──────────────────────────────────────────

    /**
     * toISOString() – ISO 8601 UTC 格式
     * e.g. "2026-02-27T06:30:00.000Z"
     */
    StdDate.prototype.toISOString = function () {
        // 从 UTC ms 手动计算 UTC 各部分
        var t = this._ms;
        var msec = ((t % 1000) + 1000) % 1000;
        t = Math.floor(t / 1000);
        var sec = ((t % 60) + 60) % 60;
        t = Math.floor(t / 60);
        var min = ((t % 60) + 60) % 60;
        t = Math.floor(t / 60);
        var hr = ((t % 24) + 24) % 24;
        var totalDays = Math.floor(t / 24);

        // 从 epoch days 计算 year/month/day
        // 算法: Civil from days (Howard Hinnant)
        var z = totalDays + 719468;
        var era = Math.floor((z >= 0 ? z : z - 146096) / 146097);
        var doe = z - era * 146097;
        var yoe = Math.floor((doe - Math.floor(doe / 1460) + Math.floor(doe / 36524) - Math.floor(doe / 146096)) / 365);
        var y = yoe + era * 400;
        var doy = doe - (365 * yoe + Math.floor(yoe / 4) - Math.floor(yoe / 100));
        var mp = Math.floor((5 * doy + 2) / 153);
        var d = doy - Math.floor((153 * mp + 2) / 5) + 1;
        var m = mp + (mp < 10 ? 3 : -9);
        if (m <= 2) y += 1;

        return pad(y, 4) + "-" + pad(m) + "-" + pad(d) +
               "T" + pad(hr) + ":" + pad(min) + ":" + pad(sec) +
               "." + pad(msec, 3) + "Z";
    };

    /**
     * toLocalString() – 本地时间的可读字符串
     * e.g. "2026-02-27 14:30:00"
     */
    StdDate.prototype.toLocalString = function () {
        var p = this._local();
        return pad(p.year, 4) + "-" + pad(p.month) + "-" + pad(p.day) +
               " " + pad(p.hours) + ":" + pad(p.minutes) + ":" + pad(p.seconds);
    };

    /**
     * toDateString() – 本地日期部分
     * e.g. "2026-02-27"
     */
    StdDate.prototype.toDateString = function () {
        var p = this._local();
        return pad(p.year, 4) + "-" + pad(p.month) + "-" + pad(p.day);
    };

    /**
     * toTimeString() – 本地时间部分
     * e.g. "14:30:00"
     */
    StdDate.prototype.toTimeString = function () {
        var p = this._local();
        return pad(p.hours) + ":" + pad(p.minutes) + ":" + pad(p.seconds);
    };

    /**
     * toString() – 完整可读字符串
     * e.g. "2026-02-27 14:30:00 +08:00"
     */
    StdDate.prototype.toString = function () {
        if (isNaN(this._ms)) return "Invalid Date";
        var p = this._local();
        var offMin = p.offset;
        var sign = offMin >= 0 ? "+" : "-";
        var absOff = Math.abs(offMin);
        var tzStr = sign + pad(Math.floor(absOff / 60)) + ":" + pad(absOff % 60);
        return this.toLocalString() + " " + tzStr;
    };

    /**
     * format(pattern) – 简易格式化
     *
     * 支持占位符:
     *   YYYY → 4位年     MM → 2位月    DD → 2位日
     *   HH   → 2位时     mm → 2位分    ss → 2位秒
     *   SSS  → 3位毫秒
     *
     * @param {string} pattern – e.g. "YYYY-MM-DD HH:mm:ss"
     * @returns {string}
     */
    StdDate.prototype.format = function format(pattern) {
        if (isNaN(this._ms)) return "Invalid Date";
        var p = this._local();
        return pattern
            .replace("YYYY", pad(p.year, 4))
            .replace("MM", pad(p.month))
            .replace("DD", pad(p.day))
            .replace("HH", pad(p.hours))
            .replace("mm", pad(p.minutes))
            .replace("ss", pad(p.seconds))
            .replace("SSS", pad(p.ms, 3));
    };

    // ── 运算支持 ─────────────────────────────────────────────

    /**
     * addMs(n) – 增加毫秒，返回新 Date
     * @param {number} n
     * @returns {StdDate}
     */
    StdDate.prototype.addMs = function (n) { return new StdDate(this._ms + n); };

    /**
     * addSeconds(n) – 增加秒
     */
    StdDate.prototype.addSeconds = function (n) { return this.addMs(n * 1000); };

    /**
     * addMinutes(n) – 增加分钟
     */
    StdDate.prototype.addMinutes = function (n) { return this.addMs(n * 60000); };

    /**
     * addHours(n) – 增加小时
     */
    StdDate.prototype.addHours = function (n) { return this.addMs(n * 3600000); };

    /**
     * addDays(n) – 增加天
     */
    StdDate.prototype.addDays = function (n) { return this.addMs(n * 86400000); };

    // ── 静态方法 ─────────────────────────────────────────────

    /**
     * Date.now() – 当前 UTC 毫秒时间戳
     * @returns {number}
     */
    StdDate.now = function () {
        return core.localTime().now;
    };

    /**
     * Date.timezone() – 当前时区信息
     * @returns {{ offset: number, name: string }}
     *   offset: UTC偏移分钟 (e.g. 480 = UTC+8)
     *   name:   偏移字符串 (e.g. "+08:00")
     */
    StdDate.timezone = function () {
        var t = core.localTime();
        return { offset: t.offset, name: t.offsetName };
    };

    /**
     * Date.parse(str) – 解析日期字符串为 UTC 毫秒时间戳
     * @param {string} str
     * @returns {number} UTC ms，无法解析返回 NaN
     */
    StdDate.parse = function (str) {
        return parseDateStr(str);
    };

    runtime.std.Date = StdDate;
})();
