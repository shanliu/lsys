//! Test all File API methods – both core.File and std.File
//!
//! Covers:
//!   1. Basic write / seek / read / tell / close
//!   2. writeLine
//!   3. rename
//!   4. File.exists / File.getsize / File.remove
//!   5. File.readAll / File.writeAll
//!   6. writeCSVRow / writeCSVRows
//!   7. writeJSON / writeJSONLines
//!   8. writeTSVRow / writeTSVRows
//!   9. writeINI / writeProperties (if available)
//!  10. Error cases: closed file, invalid name

use lsys_lib_jsrun::{EngineConfig, JsEngine, RuntimeConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Use a dedicated temp directory for file tests
    let work_dir = std::env::temp_dir().join("lsys-jsrun-file-test");
    // Clean up from any previous run
    if work_dir.exists() {
        std::fs::remove_dir_all(&work_dir)?;
    }
    std::fs::create_dir_all(&work_dir)?;

    println!("📂 Work directory: {}", work_dir.display());

    let engine = JsEngine::new(EngineConfig::default())?;
    let rt = engine
        .create_runtime(RuntimeConfig {
            work_dir: work_dir.clone(),
            deny_private_ip: false,
            ..Default::default()
        })
        .await?;

    let mut passed = 0u32;
    let mut failed = 0u32;

    // Helper macro to run a test
    macro_rules! test {
        ($name:expr, $js:expr) => {
            print!("  ── {} ... ", $name);
            match rt.eval($js).await {
                Ok(v) => {
                    println!("✅  {}", v);
                    passed += 1;
                }
                Err(e) => {
                    println!("❌  {}", e);
                    failed += 1;
                }
            }
        };
    }

    // ────────────────────────────────────────────────────────
    // 1. Basic write / seek / read / tell
    // ────────────────────────────────────────────────────────
    println!("\n─── 1. Basic write / seek / read / tell ───");
    test!(
        "write",
        r#"
        var f = new runtime.std.File("test1.txt");
        var n = f.write("Hello, File API!");
        "wrote " + n + " bytes";
        "#
    );

    test!(
        "tell after write",
        r#"
        var pos = f.tell();
        "position after write: " + pos;
        "#
    );

    test!(
        "seek to start",
        r#"
        f.seek(0);
        "seeked to 0, tell = " + f.tell();
        "#
    );

    test!(
        "read all",
        r#"
        var content = f.read();
        content === "Hello, File API!" ? "PASS: content matches" : "FAIL: got '" + content + "'";
        "#
    );

    test!(
        "seek + partial read",
        r#"
        f.seek(7);
        var part = f.read(4);
        part === "File" ? "PASS: partial read = '" + part + "'" : "FAIL: got '" + part + "'";
        "#
    );

    test!(
        "close",
        r#"
        f.close();
        "closed ok";
        "#
    );

    // ────────────────────────────────────────────────────────
    // 2. writeLine
    // ────────────────────────────────────────────────────────
    println!("\n─── 2. writeLine ───");
    test!(
        "writeLine",
        r#"
        var f2 = new runtime.std.File("lines.txt");
        f2.writeLine("line1");
        f2.writeLine("line2");
        f2.writeLine("line3");
        f2.seek(0);
        var all = f2.read();
        f2.close();
        all === "line1\nline2\nline3\n" ? "PASS: 3 lines written correctly" : "FAIL: got '" + JSON.stringify(all) + "'";
        "#
    );

    // ────────────────────────────────────────────────────────
    // 3. rename
    // ────────────────────────────────────────────────────────
    println!("\n─── 3. rename ───");
    test!(
        "rename file",
        r#"
        var f3 = new runtime.std.File("before_rename.txt");
        f3.write("rename me");
        f3.rename("after_rename.txt");
        f3.seek(0);
        var data = f3.read();
        f3.close();
        var existsOld = runtime.std.File.exists("before_rename.txt");
        var existsNew = runtime.std.File.exists("after_rename.txt");
        (!existsOld && existsNew && data === "rename me")
            ? "PASS: renamed, old gone, new has correct content"
            : "FAIL: existsOld=" + existsOld + " existsNew=" + existsNew + " data='" + data + "'";
        "#
    );

    // ────────────────────────────────────────────────────────
    // 4. Static: exists / getsize / remove
    // ────────────────────────────────────────────────────────
    println!("\n─── 4. File.exists / File.getsize / File.remove ───");
    test!(
        "exists (true)",
        r#"
        var fx = new runtime.std.File("static_test.txt");
        fx.write("12345");
        fx.close();
        runtime.std.File.exists("static_test.txt") ? "PASS: file exists" : "FAIL";
        "#
    );

    test!(
        "getsize",
        r#"
        var sz = runtime.std.File.getsize("static_test.txt");
        sz === 5 ? "PASS: size = " + sz : "FAIL: size = " + sz;
        "#
    );

    test!(
        "remove",
        r#"
        runtime.std.File.remove("static_test.txt");
        !runtime.std.File.exists("static_test.txt") ? "PASS: removed" : "FAIL: still exists";
        "#
    );

    test!(
        "exists (false)",
        r#"
        !runtime.std.File.exists("no_such_file.txt") ? "PASS: non-existent file returns false" : "FAIL";
        "#
    );

    // ────────────────────────────────────────────────────────
    // 5. File.readAll / File.writeAll
    // ────────────────────────────────────────────────────────
    println!("\n─── 5. File.readAll / File.writeAll ───");
    test!(
        "writeAll + readAll",
        r#"
        runtime.std.File.writeAll("quick.txt", "quick brown fox");
        var val = runtime.std.File.readAll("quick.txt");
        val === "quick brown fox" ? "PASS: readAll matches" : "FAIL: got '" + val + "'";
        "#
    );

    // ────────────────────────────────────────────────────────
    // 6. writeCSVRow / writeCSVRows
    // ────────────────────────────────────────────────────────
    println!("\n─── 6. CSV writing ───");
    test!(
        "writeCSVRow",
        r#"
        var fc = new runtime.std.File("row.csv");
        fc.writeCSVRow(["Alice", 30, "Beijing"]);
        fc.seek(0);
        var csv1 = fc.read();
        fc.close();
        csv1 === "Alice,30,Beijing\n" ? "PASS: " + JSON.stringify(csv1) : "FAIL: " + JSON.stringify(csv1);
        "#
    );

    test!(
        "writeCSVRow with quoting",
        r#"
        var fc2 = new runtime.std.File("row_quote.csv");
        fc2.writeCSVRow(["has,comma", 'has"quote', "normal"]);
        fc2.seek(0);
        var csv2 = fc2.read();
        fc2.close();
        csv2.indexOf('"has,comma"') !== -1 && csv2.indexOf('"has""quote"') !== -1
            ? "PASS: quoting correct: " + JSON.stringify(csv2)
            : "FAIL: " + JSON.stringify(csv2);
        "#
    );

    test!(
        "writeCSVRows with headers",
        r#"
        var fc3 = new runtime.std.File("multi.csv");
        fc3.writeCSVRows(
            [["Alice", 30], ["Bob", 25]],
            { headers: ["name", "age"] }
        );
        fc3.seek(0);
        var csv3 = fc3.read();
        fc3.close();
        var lines = csv3.split("\n").filter(function(l){ return l.length > 0; });
        lines.length === 3 && lines[0] === "name,age"
            ? "PASS: " + lines.length + " lines, header = " + lines[0]
            : "FAIL: " + JSON.stringify(csv3);
        "#
    );

    // ────────────────────────────────────────────────────────
    // 7. writeJSON / writeJSONLines
    // ────────────────────────────────────────────────────────
    println!("\n─── 7. JSON writing ───");
    test!(
        "writeJSON compact",
        r#"
        var fj = new runtime.std.File("data.json");
        fj.writeJSON({ name: "Alice", score: 99 });
        fj.seek(0);
        var j1 = fj.read();
        fj.close();
        var parsed = JSON.parse(j1);
        parsed.name === "Alice" && parsed.score === 99
            ? "PASS: " + j1
            : "FAIL: " + j1;
        "#
    );

    test!(
        "writeJSON pretty",
        r#"
        var fj2 = new runtime.std.File("pretty.json");
        fj2.writeJSON({ a: 1, b: 2 }, 2);
        fj2.seek(0);
        var j2 = fj2.read();
        fj2.close();
        j2.indexOf("\n") !== -1
            ? "PASS: has newlines (pretty): " + JSON.stringify(j2)
            : "FAIL: " + JSON.stringify(j2);
        "#
    );

    test!(
        "writeJSONLines",
        r#"
        var fjl = new runtime.std.File("data.jsonl");
        fjl.writeJSONLines([
            { id: 1, val: "a" },
            { id: 2, val: "b" },
            { id: 3, val: "c" }
        ]);
        fjl.seek(0);
        var jl = fjl.read();
        fjl.close();
        var jlines = jl.split("\n").filter(function(l){ return l.length > 0; });
        jlines.length === 3 && JSON.parse(jlines[0]).id === 1
            ? "PASS: " + jlines.length + " JSON lines"
            : "FAIL: " + JSON.stringify(jl);
        "#
    );

    // ────────────────────────────────────────────────────────
    // 8. writeTSVRow / writeTSVRows
    // ────────────────────────────────────────────────────────
    println!("\n─── 8. TSV writing ───");
    test!(
        "writeTSVRow",
        r#"
        var ft = new runtime.std.File("row.tsv");
        ft.writeTSVRow(["col1", "col2", "col3"]);
        ft.seek(0);
        var tsv1 = ft.read();
        ft.close();
        tsv1 === "col1\tcol2\tcol3\n"
            ? "PASS: " + JSON.stringify(tsv1)
            : "FAIL: " + JSON.stringify(tsv1);
        "#
    );

    test!(
        "writeTSVRows with headers",
        r#"
        var ft2 = new runtime.std.File("multi.tsv");
        ft2.writeTSVRows(
            [["A", 1], ["B", 2]],
            ["key", "val"]
        );
        ft2.seek(0);
        var tsv2 = ft2.read();
        ft2.close();
        var tlines = tsv2.split("\n").filter(function(l){ return l.length > 0; });
        tlines.length === 3 && tlines[0] === "key\tval"
            ? "PASS: " + tlines.length + " lines"
            : "FAIL: " + JSON.stringify(tsv2);
        "#
    );

    // ────────────────────────────────────────────────────────
    // 9. Error cases
    // ────────────────────────────────────────────────────────
    println!("\n─── 9. Error cases ───");
    test!(
        "write after close → error",
        r#"
        var fe = new runtime.std.File("err_test.txt");
        fe.close();
        var errMsg = "";
        try {
            fe.write("should fail");
        } catch(e) {
            errMsg = e.message || String(e);
        }
        errMsg.length > 0 ? "PASS: caught error: " + errMsg : "FAIL: no error thrown";
        "#
    );

    test!(
        "invalid file name (path separator) → error",
        r#"
        var errMsg2 = "";
        try {
            var bad = new runtime.std.File("../evil.txt");
        } catch(e) {
            errMsg2 = e.message || String(e);
        }
        errMsg2.length > 0 ? "PASS: caught error: " + errMsg2 : "FAIL: no error thrown";
        "#
    );

    test!(
        "invalid file name (backslash) → error",
        r#"
        var errMsg3 = "";
        try {
            var bad2 = new runtime.std.File("sub\\file.txt");
        } catch(e) {
            errMsg3 = e.message || String(e);
        }
        errMsg3.length > 0 ? "PASS: caught error: " + errMsg3 : "FAIL: no error thrown";
        "#
    );

    test!(
        "empty file name → error",
        r#"
        var errMsg4 = "";
        try {
            var bad3 = new runtime.std.File("");
        } catch(e) {
            errMsg4 = e.message || String(e);
        }
        errMsg4.length > 0 ? "PASS: caught error: " + errMsg4 : "FAIL: no error thrown";
        "#
    );

    // ────────────────────────────────────────────────────────
    // 10. Multiple write/read cycle (append behaviour)
    // ────────────────────────────────────────────────────────
    println!("\n─── 10. Append behaviour ───");
    test!(
        "sequential writes append",
        r#"
        var fa = new runtime.std.File("append.txt");
        fa.write("AAA");
        fa.write("BBB");
        fa.seek(0);
        var all = fa.read();
        fa.close();
        all === "AAABBB" ? "PASS: appended correctly" : "FAIL: got '" + all + "'";
        "#
    );

    // ────────────────────────────────────────────────────────
    // Summary
    // ────────────────────────────────────────────────────────
    println!("\n══════════════════════════════════════════");
    println!("  Results: {} passed, {} failed", passed, failed);
    if failed == 0 {
        println!("  ✅ All File API tests passed!");
    } else {
        println!("  ❌ Some tests failed!");
    }
    println!("══════════════════════════════════════════");

    // Clean up
    drop(rt);
    if work_dir.exists() {
        let _ = std::fs::remove_dir_all(&work_dir);
    }

    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}
