// ─────────────────────────────────────────────────────────
// runtime.std.import  –  Universal Module Loader
//
// Supports all common JS module formats:
//
//   ESM  – import/export syntax (all forms)
//            import 'mod'
//            import def from 'mod'
//            import * as ns from 'mod'
//            import { a, b as c } from 'mod'
//            import def, { a } from 'mod'
//            import def, * as ns from 'mod'
//            export default value / function / class
//            export const/let/var/function/class name
//            export { a, b as c }
//            export { a } from 'mod'          (re-export named)
//            export * from 'mod'              (re-export all)
//            export * as ns from 'mod'        (namespace re-export)
//            import('mod')                    (dynamic import)
//
//   CJS  – require() / module.exports  (natively compatible)
//   AMD  – define(deps, factory)       (polyfill provided)
//   UMD  – auto-detected via define.amd + module.exports checks
//   IIFE – runs as-is in function scope
// ─────────────────────────────────────────────────────────
(function () {
    "use strict";

    var core = runtime.core;
    var moduleRegistry = {}; // url -> exports

    // ── URL helpers ──────────────────────────────────────────────

    function resolveUrl(base, relative) {
        if (/^https?:\/\//i.test(relative)) return relative;
        var m = String(base).match(/^(https?:\/\/[^/]+)(\/.*)?$/i);
        if (!m) return relative;
        var origin = m[1];
        var basePath = m[2] || "/";
        if (relative.charAt(0) === "/") return origin + relative;
        var dir = basePath.replace(/\/[^/]*$/, "/");
        return origin + dir + relative;
    }

    // ── Source fetching ──────────────────────────────────────────

    function fetchModuleSource(specifier) {
        var cached = core.Cache.get(specifier);
        if (cached) return cached;
        var resp = runtime.std.fetch(specifier);
        if (!resp.ok) {
            throw new Error("Failed to import '" + specifier + "': HTTP " + resp.status);
        }
        var source = resp.text();
        core.Cache.set(specifier, source, 300000);
        return source;
    }

    // ── ESM → CommonJS transformer ───────────────────────────────

    /**
     * Detect whether source contains ESM-specific syntax.
     */
    function hasESMSyntax(src) {
        // Must handle minified code where export/import may appear mid-line
        // e.g. "...};export{a as b,...}"
        return /(?:^|[;{}\s])(?:import|export)\b/.test(src) || /\bimport\s*\(/.test(src);
    }

    /**
     * Protect string literals, template literals, and comments
     * from being matched by our import/export regexes.
     * Returns { safe, slots } where slots[i] is the original text.
     */
    function protectStrings(src) {
        var slots = [];
        var safe = src.replace(
            // Order matters: block comments, line comments,
            // template literals (no nested ${}), double-quoted,
            // single-quoted strings.
            /\/\*[\s\S]*?\*\/|\/\/[^\n]*|`(?:[^`\\]|\\.)*`|"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'/g,
            function (m) {
                slots.push(m);
                return "\x01" + (slots.length - 1) + "\x01";
            }
        );
        return { safe: safe, slots: slots };
    }

    function restoreStrings(safe, slots) {
        return safe.replace(/\x01(\d+)\x01/g, function (_, i) {
            return slots[+i];
        });
    }

    /**
     * Parse a comma-separated named-import/export list like
     *   "a, b as c, default as d"
     * and return an array of { local, exported } pairs.
     */
    function parseNamedList(spec) {
        var result = [];
        spec.split(",").forEach(function (item) {
            item = item.trim();
            if (!item) return;
            var m = item.match(/^([A-Za-z_$][\w$]*)\s+as\s+([A-Za-z_$][\w$]*)$/);
            result.push(m ? { local: m[1], exported: m[2] } : { local: item, exported: item });
        });
        return result;
    }

    /**
     * Full ESM-to-CJS transformation.
     *
     * Returns { redirect: null, source: string }.
     */
    function transformESM(source) {
        var ps = protectStrings(source);
        var safe = ps.safe;
        var slots = ps.slots;

        var impCnt = 0;      // counter for _impN temp vars
        var reCnt  = 0;      // counter for _reN temp vars (re-exports)
        var namedDecls = []; // names from `export const/function/class` → append at end
        var hasDefault = false;
        var defaultName = null; // non-null → named func/class default
        var hasAnyExport = false;

        // ── 1. Static import declarations ────────────────────────

        // 1a.  import 'spec'  (side-effect only)
        safe = safe.replace(
            /^[ \t]*import\s+(\x01\d+\x01)[ \t]*;?[ \t]*/gm,
            function (_, spec) {
                return "runtime.std.import(" + spec + ");\n";
            }
        );

        // 1b.  import * as ns from 'spec'
        safe = safe.replace(
            /^[ \t]*import\s+\*\s+as\s+([A-Za-z_$][\w$]*)\s+from\s+(\x01\d+\x01)[ \t]*;?[ \t]*/gm,
            function (_, ns, spec) {
                return "var " + ns + " = runtime.std.import(" + spec + ");\n";
            }
        );

        // 1c.  import def, * as ns from 'spec'
        safe = safe.replace(
            /^[ \t]*import\s+([A-Za-z_$][\w$]*)\s*,\s*\*\s+as\s+([A-Za-z_$][\w$]*)\s+from\s+(\x01\d+\x01)[ \t]*;?[ \t]*/gm,
            function (_, def, ns, spec) {
                var i = impCnt++;
                return (
                    "var _imp" + i + " = runtime.std.import(" + spec + ");\n" +
                    "var " + def + " = _imp" + i + ".__esModule ? _imp" + i + ".default : _imp" + i + ";\n" +
                    "var " + ns  + " = _imp" + i + ";\n"
                );
            }
        );

        // 1d.  import def, { a, b as c } from 'spec'
        safe = safe.replace(
            /^[ \t]*import\s+([A-Za-z_$][\w$]*)\s*,\s*\{([^}]*)\}\s+from\s+(\x01\d+\x01)[ \t]*;?[ \t]*/gm,
            function (_, def, named, spec) {
                var i = impCnt++;
                var out =
                    "var _imp" + i + " = runtime.std.import(" + spec + ");\n" +
                    "var " + def + " = _imp" + i + ".__esModule ? _imp" + i + ".default : _imp" + i + ";\n";
                parseNamedList(named).forEach(function (n) {
                    out += "var " + n.exported + " = _imp" + i + "." + n.local + ";\n";
                });
                return out;
            }
        );

        // 1e.  import { a, b as c } from 'spec'
        safe = safe.replace(
            /^[ \t]*import\s+\{([^}]*)\}\s+from\s+(\x01\d+\x01)[ \t]*;?[ \t]*/gm,
            function (_, named, spec) {
                var i = impCnt++;
                var out = "var _imp" + i + " = runtime.std.import(" + spec + ");\n";
                parseNamedList(named).forEach(function (n) {
                    out += "var " + n.exported + " = _imp" + i + "." + n.local + ";\n";
                });
                return out;
            }
        );

        // 1f.  import def from 'spec'
        safe = safe.replace(
            /^[ \t]*import\s+([A-Za-z_$][\w$]*)\s+from\s+(\x01\d+\x01)[ \t]*;?[ \t]*/gm,
            function (_, def, spec) {
                var i = impCnt++;
                return (
                    "var _imp" + i + " = runtime.std.import(" + spec + ");\n" +
                    "var " + def + " = _imp" + i + ".__esModule ? _imp" + i + ".default : _imp" + i + ";\n"
                );
            }
        );

        // ── 2. Export re-exports (must run before local exports) ─

        // 2a.  export { a, b as c } from 'spec'
        safe = safe.replace(
            /(?:^|(?<=[;{}\n]))[ \t]*export\s*\{([^}]*)\}\s*from\s+(\x01\d+\x01)[ \t]*;?[ \t]*/gm,
            function (_, named, spec) {
                hasAnyExport = true;
                var i = reCnt++;
                var out = "var _re" + i + " = runtime.std.import(" + spec + ");\n";
                parseNamedList(named).forEach(function (n) {
                    out += "module.exports." + n.exported + " = _re" + i + "." + n.local + ";\n";
                });
                return out;
            }
        );

        // 2b.  export * as ns from 'spec'  (namespace re-export)
        safe = safe.replace(
            /(?:^|(?<=[;{}\n]))[ \t]*export\s*\*\s+as\s+([A-Za-z_$][\w$]*)\s+from\s+(\x01\d+\x01)[ \t]*;?[ \t]*/gm,
            function (_, ns, spec) {
                hasAnyExport = true;
                return "module.exports." + ns + " = runtime.std.import(" + spec + ");\n";
            }
        );

        // 2c.  export * from 'spec'  (spread re-export, skip default)
        safe = safe.replace(
            /(?:^|(?<=[;{}\n]))[ \t]*export\s*\*\s+from\s+(\x01\d+\x01)[ \t]*;?[ \t]*/gm,
            function (_, spec) {
                hasAnyExport = true;
                var i = reCnt++;
                return (
                    "var _re" + i + " = runtime.std.import(" + spec + ");\n" +
                    "for (var _k" + i + " in _re" + i + ") {\n" +
                    "  if (_k" + i + " !== \"default\" &&\n" +
                    "      Object.prototype.hasOwnProperty.call(_re" + i + ", _k" + i + "))\n" +
                    "    module.exports[_k" + i + "] = _re" + i + "[_k" + i + "];\n" +
                    "}\n"
                );
            }
        );

        // ── 3. Local export declarations ─────────────────────────

        // 3a.  export { a, b as c }
        safe = safe.replace(
            /(?:^|(?<=[;{}\n]))[ \t]*export\s*\{([^}]*)\}[ \t]*;?[ \t]*/gm,
            function (_, named) {
                hasAnyExport = true;
                var out = "";
                parseNamedList(named).forEach(function (n) {
                    out += "module.exports." + n.exported + " = " + n.local + ";\n";
                });
                return out;
            }
        );

        // 3b.  export default function/class Name  (named – hoistable)
        safe = safe.replace(
            /^[ \t]*export\s+default\s+((?:async\s+)?function\s*\*?|class)\s+([A-Za-z_$][\w$]*)/gm,
            function (_, kw, name) {
                hasAnyExport = true;
                hasDefault = true;
                defaultName = name;
                // Keep the declaration, assignment added in suffix
                return kw.trimRight() + " " + name;
            }
        );

        // 3c.  export default function/class  (anonymous)
        safe = safe.replace(
            /^[ \t]*export\s+default\s+((?:async\s+)?function\s*\*?|class)(?!\s*[A-Za-z_$\w])/gm,
            function (_, kw) {
                hasAnyExport = true;
                hasDefault = true;
                defaultName = null;
                return "module.exports[\"default\"] = module.exports.default = " + kw.trimRight();
            }
        );

        // 3d.  export default <expression>  (anything else)
        safe = safe.replace(
            /^[ \t]*export\s+default\s+/gm,
            function () {
                hasAnyExport = true;
                hasDefault = true;
                defaultName = null;
                return "module.exports[\"default\"] = module.exports.default = ";
            }
        );

        // 3e.  export const/let/var name = ...
        safe = safe.replace(
            /^[ \t]*export\s+(const|let|var)\s+([A-Za-z_$][\w$]*)/gm,
            function (_, kw, name) {
                hasAnyExport = true;
                namedDecls.push(name);
                return kw + " " + name;
            }
        );

        // 3f.  export async function / export function* / export function
        safe = safe.replace(
            /^[ \t]*export\s+((?:async\s+)?function\s*\*?)\s+([A-Za-z_$][\w$]*)/gm,
            function (_, kw, name) {
                hasAnyExport = true;
                namedDecls.push(name);
                return kw.trimRight() + " " + name;
            }
        );

        // 3g.  export class Name
        safe = safe.replace(
            /^[ \t]*export\s+(class)\s+([A-Za-z_$][\w$]*)/gm,
            function (_, kw, name) {
                hasAnyExport = true;
                namedDecls.push(name);
                return kw + " " + name;
            }
        );

        // ── 4. Dynamic import() ──────────────────────────────────
        safe = safe.replace(
            /\bimport\s*\(\s*(\x01\d+\x01)\s*\)/g,
            function (_, spec) {
                return "Promise.resolve(runtime.std.import(" + spec + "))";
            }
        );

        // ── 5. Restore string placeholders ──────────────────────
        var result = restoreStrings(safe, slots);

        // ── 6. Wrap with module.exports preamble / suffix ────────
        if (hasAnyExport) {
            var prefix = "module.exports = module.exports || {};\nmodule.exports.__esModule = true;\n";
            var suffix = "";
            if (hasDefault && defaultName) {
                suffix += "\nmodule.exports[\"default\"] = module.exports.default = " + defaultName + ";";
            }
            for (var ni = 0; ni < namedDecls.length; ni++) {
                suffix += "\nmodule.exports." + namedDecls[ni] + " = " + namedDecls[ni] + ";";
            }
            result = prefix + result + suffix;
        }

        return { redirect: null, source: result };
    }

    /**
     * Full module transformation pipeline.
     *
     * Fast-path 1: pure redirect file (esm.sh entry stubs)
     *   export * from '/path'
     *
     * Fast-path 2: no ESM syntax → return as-is (CJS/UMD/IIFE)
     *
     * Otherwise: full ESM → CJS transformation.
     */
    function transformModule(source) {
        // Fast path: esm.sh-style single-line redirect
        var redir =
            source.match(/^\s*\/\*[\s\S]*?\*\/\s*export\s+\*\s+from\s+["']([^"']+)["']\s*;?\s*$/) ||
            source.match(/^\s*export\s+\*\s+from\s+["']([^"']+)["']\s*;?\s*$/);
        if (redir) {
            return { redirect: redir[1], source: null };
        }

        // Fast path: no ESM syntax at all (CJS, UMD, IIFE)
        if (!hasESMSyntax(source)) {
            return { redirect: null, source: source };
        }

        return transformESM(source);
    }

    // ── Sandbox helpers ──────────────────────────────────────────

    /**
     * Create a require() function for use inside the sandbox.
     * Relative specifiers are resolved against baseUrl.
     */
    function makeRequire(baseUrl) {
        return function require(specifier) {
            var url = /^\.{0,2}\//.test(specifier)
                ? resolveUrl(baseUrl, specifier)
                : specifier;
            return runtime.std.import(url);
        };
    }

    /**
     * Create an AMD define() polyfill for use inside the sandbox.
     * Supports:
     *   define(factory)
     *   define(deps, factory)
     *   define(id, deps, factory)    ← id is ignored
     */
    function makeDefine(baseUrl, modObj, exportsObj) {
        function define(id, deps, factory) {
            // Shift arguments: define(factory) or define(deps, factory)
            if (typeof id === "function") {
                factory = id; deps = [];
            } else if (Array.isArray(id)) {
                factory = deps; deps = id;
            }
            if (!Array.isArray(deps)) deps = [];

            var req = makeRequire(baseUrl);
            var resolved = deps.map(function (d) {
                if (d === "require")  return req;
                if (d === "exports")  return exportsObj;
                if (d === "module")   return modObj;
                return req(d);
            });

            var result = factory.apply(null, resolved);
            if (result !== undefined) modObj.exports = result;
        }
        define.amd = {};
        return define;
    }

    // ── Public API ───────────────────────────────────────────────

    /**
     * runtime.std.import(specifier)
     *
     * Load a JS module from an absolute URL (or a bare specifier for
     * modules already registered in the local registry).
     *
     * The module is executed in an isolated CommonJS-like sandbox that
     * also provides:
     *   require(spec)     – synchronous CJS/ESM require
     *   define(...)       – AMD define() polyfill
     *   __filename        – the module's own URL
     *   __dirname         – the directory portion of the URL
     *
     * @param {string} specifier
     * @returns {object} The module's exports.
     */
    runtime.std.import = function importModule(specifier) {
        if (moduleRegistry[specifier]) {
            return moduleRegistry[specifier];
        }

        var source = fetchModuleSource(specifier);
        var converted = transformModule(source);

        if (converted.redirect) {
            var redirectUrl = resolveUrl(specifier, converted.redirect);
            var redirectExports = runtime.std.import(redirectUrl);
            moduleRegistry[specifier] = redirectExports;
            return redirectExports;
        }

        source = converted.source;

        var mod  = { exports: {} };
        var exp  = mod.exports;
        var req  = makeRequire(specifier);
        var def  = makeDefine(specifier, mod, exp);
        var dir  = specifier.replace(/\/[^/]*$/, "/");

        var fn = new Function(
            "module", "exports", "runtime",
            "require", "define",
            "__filename", "__dirname",
            source
        );
        fn(mod, exp, runtime, req, def, specifier, dir);

        moduleRegistry[specifier] = mod.exports;
        return mod.exports;
    };
})();
