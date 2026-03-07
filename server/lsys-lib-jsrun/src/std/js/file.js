// ─────────────────────────────────────────────────────────
// runtime.std.File – Sandboxed file I/O (wraps core.File)
//
// 文件操作限制在 runtime 的 work_dir 内，文件名不允许
// 包含路径分隔符，防止目录穿越。runtime 退出时自动关闭
// 所有打开的文件。
//
// 实例方法:
//   var f = new runtime.std.File(name)
//   f.write(data)       → 写入字符串，返回写入字节数
//   f.writeLine(line)   → 写入一行（自动追加换行符）
//   f.read(size?)       → 读取内容，不传 size 读取全部
//   f.seek(offset)      → 定位到绝对位置
//   f.tell()            → 获取当前文件指针位置
//   f.close()           → 关闭文件
//   f.rename(newName)   → 重命名文件
//   f.localSync()       → 同步文件 (调用宿主 file_sync_handler)
//
// 格式化写入:
//   f.writeCSVRow(fields, sep?)       → 写入一行 CSV
//   f.writeCSVRows(rows, sep?)        → 写入多行 CSV
//   f.writeJSON(obj, indent?)         → 写入 JSON
//   f.writeJSONLines(arr)             → 写入 JSON Lines (NDJSON)
//   f.writeTSVRow(fields)             → 写入一行 TSV
//   f.writeTSVRows(rows)              → 写入多行 TSV
//   f.writeINI(sections)              → 写入 INI 格式
//   f.writeProperties(obj)            → 写入 key=value 格式
//
// 静态方法:
//   runtime.std.File.exists(name)  → 文件是否存在
//   runtime.std.File.getsize(name) → 文件大小(字节)
//   runtime.std.File.remove(name)  → 删除文件
//   runtime.std.File.readAll(name) → 便捷：读取整个文件内容
//   runtime.std.File.writeAll(name, data) → 便捷：覆盖写入整个文件
// ─────────────────────────────────────────────────────────
(function () {
    "use strict";

    var CoreFile = runtime.core.File;

    /**
     * runtime.std.File(name) – 打开或创建一个文件
     *
     * 底层调用 core.File (Rust JsFile)，并在其基础上
     * 提供额外的便捷方法。
     *
     * @param {string} name – 文件名 (不含路径分隔符)
     */
    function StdFile(name) {
        if (!(this instanceof StdFile)) {
            return new StdFile(name);
        }
        this._file = new CoreFile(name);
        this.name = name;
    }

    // ── 实例方法 ─────────────────────────────────────────────

    /**
     * write(data) – 写入字符串
     * @param {string} data
     * @returns {number} 写入的字节数
     */
    StdFile.prototype.write = function write(data) {
        return this._file.write(data);
    };

    /**
     * read(size?) – 读取内容
     * @param {number} [size] – 读取字节数，不传则读取到末尾
     * @returns {string} 文件内容
     */
    StdFile.prototype.read = function read(size) {
        if (arguments.length > 0) {
            return this._file.read(size);
        }
        return this._file.read();
    };

    /**
     * seek(offset) – 定位到绝对位置
     * @param {number} offset
     * @returns {number} 新位置
     */
    StdFile.prototype.seek = function seek(offset) {
        return this._file.seek(offset);
    };

    /**
     * tell() – 获取当前文件指针位置
     * @returns {number}
     */
    StdFile.prototype.tell = function tell() {
        return this._file.tell();
    };

    /**
     * close() – 关闭文件
     */
    StdFile.prototype.close = function close() {
        this._file.close();
    };

    /**
     * rename(newName) – 重命名文件
     * @param {string} newName
     */
    StdFile.prototype.rename = function rename(newName) {
        this._file.rename(newName);
        this.name = newName;
    };

    /**
     * localSync() – 同步文件到外部存储
     *
     * 调用宿主通过 RuntimeConfig.file_sync_handler 配置的
     * 异步处理函数，将文件完整路径、工作目录、命名空间等
     * 环境信息传递给宿主实现。
     *
     * 如果宿主未配置 file_sync_handler 则抛出异常。
     *
     * @returns {*} 宿主处理函数返回的 JSON 解析结果
     */
    StdFile.prototype.localSync = function localSync() {
        var json = this._file.local_sync();
        return JSON.parse(json);
    };

    // ── 格式化写入方法 ───────────────────────────────────────

    /**
     * writeLine(line) – 写入一行（自动追加 \n）
     * @param {string} line
     * @returns {number} 写入的字节数
     */
    StdFile.prototype.writeLine = function writeLine(line) {
        return this._file.write(line + "\n");
    };

    // ── CSV 内部工具 ─────────────────────────────────────────

    /**
     * 将单个字段转为 RFC 4180 兼容的 CSV 字段。
     * 当字段包含分隔符、引号、换行时自动加双引号转义。
     */
    function csvEscapeField(val, sep) {
        var s = val === null || val === undefined ? "" : String(val);
        if (
            s.indexOf(sep) !== -1 ||
            s.indexOf('"') !== -1 ||
            s.indexOf("\n") !== -1 ||
            s.indexOf("\r") !== -1
        ) {
            return '"' + s.replace(/"/g, '""') + '"';
        }
        return s;
    }

    function fieldsToCSV(fields, sep) {
        var parts = [];
        for (var i = 0; i < fields.length; i++) {
            parts.push(csvEscapeField(fields[i], sep));
        }
        return parts.join(sep) + "\n";
    }

    /**
     * writeCSVRow(fields, sep?) – 写入一行 CSV
     *
     * @param {Array} fields – 字段数组
     * @param {string} [sep=","] – 分隔符，默认逗号
     * @returns {number} 写入的字节数
     */
    StdFile.prototype.writeCSVRow = function writeCSVRow(fields, sep) {
        sep = sep || ",";
        return this._file.write(fieldsToCSV(fields, sep));
    };

    /**
     * writeCSVRows(rows, options?) – 写入多行 CSV
     *
     * @param {Array<Array>} rows – 二维数组
     * @param {object} [options]
     * @param {string} [options.sep=","] – 分隔符
     * @param {Array<string>} [options.headers] – 表头，写在第一行
     * @returns {number} 总写入字节数
     */
    StdFile.prototype.writeCSVRows = function writeCSVRows(rows, options) {
        options = options || {};
        var sep = options.sep || ",";
        var total = 0;
        if (options.headers && Array.isArray(options.headers)) {
            total += this._file.write(fieldsToCSV(options.headers, sep));
        }
        for (var i = 0; i < rows.length; i++) {
            total += this._file.write(fieldsToCSV(rows[i], sep));
        }
        return total;
    };

    // ── TSV ──────────────────────────────────────────────────

    function tsvEscapeField(val) {
        var s = val === null || val === undefined ? "" : String(val);
        // TSV 不使用引号，将 tab/换行替换为空格
        return s.replace(/[\t\r\n]/g, " ");
    }

    function fieldsToTSV(fields) {
        var parts = [];
        for (var i = 0; i < fields.length; i++) {
            parts.push(tsvEscapeField(fields[i]));
        }
        return parts.join("\t") + "\n";
    }

    /**
     * writeTSVRow(fields) – 写入一行 TSV
     *
     * @param {Array} fields
     * @returns {number} 写入的字节数
     */
    StdFile.prototype.writeTSVRow = function writeTSVRow(fields) {
        return this._file.write(fieldsToTSV(fields));
    };

    /**
     * writeTSVRows(rows, headers?) – 写入多行 TSV
     *
     * @param {Array<Array>} rows
     * @param {Array<string>} [headers] – 可选表头
     * @returns {number} 总写入字节数
     */
    StdFile.prototype.writeTSVRows = function writeTSVRows(rows, headers) {
        var total = 0;
        if (headers && Array.isArray(headers)) {
            total += this._file.write(fieldsToTSV(headers));
        }
        for (var i = 0; i < rows.length; i++) {
            total += this._file.write(fieldsToTSV(rows[i]));
        }
        return total;
    };

    // ── JSON ─────────────────────────────────────────────────

    /**
     * writeJSON(obj, indent?) – 写入 JSON
     *
     * @param {*} obj – 任意可序列化的值
     * @param {number} [indent=0] – 缩进空格数，0 表示紧凑格式
     * @returns {number} 写入的字节数
     */
    StdFile.prototype.writeJSON = function writeJSON(obj, indent) {
        var str =
            indent && indent > 0
                ? JSON.stringify(obj, null, indent)
                : JSON.stringify(obj);
        return this._file.write(str);
    };

    /**
     * writeJSONLines(arr) – 写入 JSON Lines (NDJSON) 格式
     *
     * 每个数组元素序列化为一行 JSON，以 \n 分隔。
     * 适用于流式处理和大数据导出。
     *
     * @param {Array} arr – 对象数组
     * @returns {number} 总写入字节数
     */
    StdFile.prototype.writeJSONLines = function writeJSONLines(arr) {
        var total = 0;
        for (var i = 0; i < arr.length; i++) {
            total += this._file.write(JSON.stringify(arr[i]) + "\n");
        }
        return total;
    };

    // ── 静态方法 ─────────────────────────────────────────────

    /**
     * File.exists(name) – 文件是否存在
     * @param {string} name
     * @returns {boolean}
     */
    StdFile.exists = function exists(name) {
        return CoreFile.exists(name);
    };

    /**
     * File.getsize(name) – 获取文件大小（字节）
     * @param {string} name
     * @returns {number}
     */
    StdFile.getsize = function getsize(name) {
        return CoreFile.getsize(name);
    };

    /**
     * File.remove(name) – 删除文件
     * @param {string} name
     */
    StdFile.remove = function remove(name) {
        CoreFile.remove(name);
    };

    /**
     * File.readAll(name) – 便捷方法：读取整个文件内容
     *
     * 自动打开、读取、关闭文件。
     *
     * @param {string} name
     * @returns {string} 文件全部内容
     */
    StdFile.readAll = function readAll(name) {
        var f = new CoreFile(name);
        try {
            f.seek(0);
            return f.read();
        } finally {
            f.close();
        }
    };

    /**
     * File.writeAll(name, data) – 便捷方法：覆盖写入整个文件
     *
     * 自动打开、定位到开头、写入、关闭文件。
     *
     * @param {string} name
     * @param {string} data
     * @returns {number} 写入的字节数
     */
    StdFile.writeAll = function writeAll(name, data) {
        var f = new CoreFile(name);
        try {
            f.seek(0);
            return f.write(data);
        } finally {
            f.close();
        }
    };

    runtime.std.File = StdFile;
})();
