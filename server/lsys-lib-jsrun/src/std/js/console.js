// ─────────────────────────────────────────────────────────
// runtime.std.console  –  Based on core.log(level, msg)
//
// 日志等级由 Rust 定义，通过 core.LogLevel 传入 JS:
//   core.LogLevel.TRACE = 0
//   core.LogLevel.DEBUG = 1
//   core.LogLevel.INFO  = 2
//   core.LogLevel.WARN  = 3
//   core.LogLevel.ERROR = 4
// ─────────────────────────────────────────────────────────
(function () {
    "use strict";

    var core = runtime.core;
    var Level = core.LogLevel;
    var timers = {};

    function formatArgs(args) {
        return args
            .map(function (a) {
                if (a === null) return "null";
                if (a === undefined) return "undefined";
                if (typeof a === "object") {
                    try {
                        return JSON.stringify(a, null, 2);
                    } catch (_) {
                        return String(a);
                    }
                }
                return String(a);
            })
            .join(" ");
    }

    runtime.std.console = {
        log: function () {
            core.log(Level.INFO, formatArgs(Array.prototype.slice.call(arguments)));
        },
        info: function () {
            core.log(Level.INFO, formatArgs(Array.prototype.slice.call(arguments)));
        },
        debug: function () {
            core.log(Level.DEBUG, formatArgs(Array.prototype.slice.call(arguments)));
        },
        trace: function () {
            core.log(Level.TRACE, formatArgs(Array.prototype.slice.call(arguments)));
        },
        warn: function () {
            core.log(Level.WARN, formatArgs(Array.prototype.slice.call(arguments)));
        },
        error: function () {
            core.log(Level.ERROR, formatArgs(Array.prototype.slice.call(arguments)));
        },
        time: function (label) {
            label = label || "default";
            timers[label] = core.localTime().now;
        },
        timeEnd: function (label) {
            label = label || "default";
            if (timers[label] !== undefined) {
                var elapsed = core.localTime().now - timers[label];
                core.log(Level.DEBUG, "[TIMER] " + label + ": " + elapsed.toFixed(3) + "ms");
                delete timers[label];
            } else {
                core.log(Level.WARN, "[TIMER] Timer '" + label + "' does not exist");
            }
        },
    };
})();
