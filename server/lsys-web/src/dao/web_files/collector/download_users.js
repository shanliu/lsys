// lsys-lib-jsrun example:
// - 分页下载 users（最多 100 页，下一页为空则停止）
// - JSON → CSV
// - 使用已绑定的 runtime.std.File 写入 users.csv
// - 最后调用 file.localSync() 同步
(function () {
  var base = 'https://jsonplaceholder.typicode.com/users';
  var limit = 2;
  var maxPages = 5;

  if (typeof runtime === 'undefined' || !runtime.std) {
    throw new Error('runtime.std is required (lsys-lib-jsrun host runtime only).');
  }

  var std = runtime.std;

  function normalizeCell(v) {
    if (v === null || v === undefined) return '';
    if (typeof v === 'object') return JSON.stringify(v);
    return String(v);
  }

  function buildHeaders(items) {
    var set = new Set();
    for (var i = 0; i < items.length; i++) {
      var obj = items[i];
      for (var k in obj) {
        if (Object.prototype.hasOwnProperty.call(obj, k)) set.add(k);
      }
    }
    return Array.from(set);
  }

  function toRows(items, headers) {
    var rows = [];
    for (var i = 0; i < items.length; i++) {
      var obj = items[i];
      var row = [];
      for (var h = 0; h < headers.length; h++) {
        row.push(normalizeCell(obj[headers[h]]));
      }
      rows.push(row);
    }
    return rows;
  }

  function log() {
    if (std.console && typeof std.console.log === 'function') {
      std.console.log.apply(std.console, arguments);
      return;
    }
  }

  if (!std.fetch || !std.File) {
    throw new Error('runtime.std.fetch and runtime.std.File are required.');
  }

  var all = [];

  // 先打开文件（按你的要求）
  // 注意：底层 File 打开时不会 truncate；这里采取“先打开→关闭→删除→重建”的方式确保覆盖。
  var f = new std.File('users.csv');
  try {
    try { f.close(); } catch (e) {}
    if (std.File.exists('users.csv')) {
      std.File.remove('users.csv');
    }
  } catch (e) {
    // ignore
  }

  // 重新打开文件，并保持打开状态直到写完/同步
  f = new std.File('users.csv');
  try {
    // 分页获取（最多 100 页，或下一页为空停止）
    for (var page = 1; page <= maxPages; page++) {
      var url = base + '?_page=' + page + '&_limit=' + limit;
      var resp = std.fetch(url);
      if (!resp.ok) throw new Error('HTTP ' + resp.status + ' on page ' + page);
      var data = resp.json();
      if (!Array.isArray(data) || data.length === 0) {
        log('No more data; stopping at page', page);
        break;
      }
      log('Fetched page', page, '->', data.length, 'items');
      all = all.concat(data);
      if (data.length < limit) break;
    }

    if (all.length === 0) {
      log('No data fetched.');
      return;
    }

    // JSON → CSV → 写入
    var headers = buildHeaders(all);
    var rows = toRows(all, headers);
    f.writeCSVRows(rows, { headers: headers });

    // 最后调用 localSync 同步
    var syncResult = f.localSync();
    log('localSync result:', syncResult);
  } finally {
    try { f.close(); } catch (e) {}
  }
})();
