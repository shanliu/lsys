//! Test loading 5 external JS libraries via runtime.std.import
//!
//! Libraries tested:
//!   1. dayjs       – UMD (jsDelivr)
//!   2. json2csv    – UMD (jsDelivr)
//!   3. mathjs      – UMD (cdnjs)
//!   4. xlsx        – UMD (SheetJS CDN)
//!   5. uuid        – ESM (esm.sh)

use lsys_lib_jsrun::{EngineConfig, JsEngine, RuntimeConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let engine = JsEngine::new(EngineConfig::default())?;
    let rt = engine
        .create_runtime(RuntimeConfig {
            deny_private_ip: false,
            ..Default::default()
        })
        .await?;

    // 初始化浏览器兼容全局变量
    rt.eval("runtime.std.initGlobalsEnv();").await?;

    // ────────────────────────────────────────────────────────
    // 1. dayjs  (UMD)
    // ────────────────────────────────────────────────────────
    println!("─── 1. dayjs ───");
    match rt
        .eval(
            r#"
            var dayjs = runtime.std.import("https://cdn.jsdelivr.net/npm/dayjs@1.11.13/dayjs.min.js");
            "dayjs loaded: " + typeof dayjs + " | now = " + dayjs().format("YYYY-MM-DD HH:mm:ss");
            "#,
        )
        .await
    {
        Ok(v) => println!("  ✅ {}", v),
        Err(e) => println!("  ❌ {}", e),
    }

    // ────────────────────────────────────────────────────────
    // 2. json2csv  (UMD)
    // ────────────────────────────────────────────────────────
    println!("─── 2. json2csv ───");
    match rt
        .eval(
            r#"
            var json2csv = runtime.std.import("https://cdn.jsdelivr.net/npm/json2csv@4.2.1");
            var Parser = json2csv.Parser || json2csv.parse || json2csv;
            var data = [
                { name: "Alice", age: 30, city: "Beijing" },
                { name: "Bob",   age: 25, city: "Shanghai" }
            ];
            var result;
            if (typeof Parser === "function" && Parser.prototype && Parser.prototype.parse) {
                var parser = new Parser({ fields: ["name", "age", "city"] });
                result = parser.parse(data);
            } else if (typeof json2csv.parse === "function") {
                result = json2csv.parse(data, { fields: ["name", "age", "city"] });
            } else {
                result = "Parser type: " + typeof Parser + " | keys: " + Object.keys(json2csv).join(",");
            }
            result;
            "#,
        )
        .await
    {
        Ok(v) => println!("  ✅ {}", v),
        Err(e) => println!("  ❌ {}", e),
    }

    // ────────────────────────────────────────────────────────
    // 3. mathjs  (UMD)
    // ────────────────────────────────────────────────────────
    println!("─── 3. mathjs ───");
    match rt
        .eval(
            r#"
            var math = runtime.std.import("https://cdnjs.cloudflare.com/ajax/libs/mathjs/12.4.0/math.min.js");
            var results = [];
            results.push("evaluate 2+3*4 = " + math.evaluate("2 + 3 * 4"));
            results.push("sqrt(144) = " + math.sqrt(144));
            results.push("pi = " + math.pi);
            results.push("sin(pi/2) = " + math.sin(math.pi / 2));
            results.push("matrix det = " + math.det([[1,2],[3,4]]));
            results.join(" | ");
            "#,
        )
        .await
    {
        Ok(v) => println!("  ✅ {}", v),
        Err(e) => println!("  ❌ {}", e),
    }

    // ────────────────────────────────────────────────────────
    // 4. xlsx / SheetJS  (UMD)
    // ────────────────────────────────────────────────────────
    println!("─── 4. xlsx (SheetJS) ───");
    match rt
        .eval(
            r#"
            var XLSX = runtime.std.import("https://cdn.sheetjs.com/xlsx-0.20.0/package/dist/xlsx.full.min.js");
            var ws_data = [
                ["Name", "Age", "City"],
                ["Alice", 30, "Beijing"],
                ["Bob",   25, "Shanghai"]
            ];
            var ws = XLSX.utils.aoa_to_sheet(ws_data);
            var csv = XLSX.utils.sheet_to_csv(ws);
            "XLSX loaded, version=" + (XLSX.version || "?") + "\nCSV output:\n" + csv;
            "#,
        )
        .await
    {
        Ok(v) => println!("  ✅ {}", v),
        Err(e) => println!("  ❌ {}", e),
    }

    // ────────────────────────────────────────────────────────
    // 5. uuid  (ESM via esm.sh)
    // ────────────────────────────────────────────────────────
    println!("─── 5. uuid (esm.sh) ───");
    match rt
        .eval(
            r#"
            var uuid = runtime.std.import("https://esm.sh/uuid");
            var keys = Object.keys(uuid).join(", ");
            var id;
            if (typeof uuid.v4 === "function") {
                id = uuid.v4();
            } else if (typeof uuid.default === "function") {
                id = uuid.default();
            } else {
                id = "no v4 found";
            }
            "uuid keys: [" + keys + "] | v4() = " + id;
            "#,
        )
        .await
    {
        Ok(v) => println!("  ✅ {}", v),
        Err(e) => println!("  ❌ {}", e),
    }

    println!("\n✅ All import tests completed!");
    Ok(())
}
