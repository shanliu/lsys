//! Example: using `check_js_syntax` to validate JavaScript source code
//! before submitting it to a full runtime.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example syntax_check
//! ```

use lsys_lib_jsrun::check_js_syntax;

fn main() {
    println!("=== JS Syntax Checker Examples ===\n");

    // ── 1. Valid scripts ────────────────────────────────────────────────────
    let valid_scripts = [
        ("simple expression", "1 + 2;"),
        ("variable declaration", "let x = 42; const y = 'hello';"),
        ("function definition", "function greet(name) { return 'hi ' + name; }"),
        ("arrow function", "const add = (a, b) => a + b;"),
        (
            "async / await",
            "async function load() { const r = await fetch('/api'); return r; }",
        ),
        (
            "destructuring",
            "const { a, b: c, ...rest } = { a: 1, b: 2, d: 3 };",
        ),
        (
            "template literal",
            "const msg = `hello ${name}, today is ${new Date()}`;",
        ),
        (
            "class definition",
            "class Foo extends Bar { constructor() { super(); this.x = 1; } }",
        ),
    ];

    println!("── Valid scripts ──");
    for (label, code) in &valid_scripts {
        match check_js_syntax(code) {
            Ok(()) => println!("  ✓ {label}"),
            Err(e) => println!("  ✗ {label} — unexpected error: {e}"),
        }
    }

    // ── 2. Scripts with syntax errors ──────────────────────────────────────
    let invalid_scripts = [
        ("unexpected token", "let x = ;"),
        ("unclosed bracket", "let arr = [1, 2, 3"),
        ("unclosed string", r#"let s = "hello"#),
        ("unclosed brace", "function foo() {"),
        ("invalid assignment", "1 = 2;"),
        ("const without init", "const x;"),
        (
            "duplicate parameter (strict)",
            "'use strict'; function f(a, a) {}",
        ),
        (
            "multi-line error",
            "let x = 1;\nlet y = 2;\nlet z = ;\nlet w = 4;",
        ),
    ];

    println!("\n── Scripts with syntax errors ──");
    for (label, code) in &invalid_scripts {
        match check_js_syntax(code) {
            Ok(()) => println!("  ✗ {label} — expected error but got Ok"),
            Err(e) => println!("  ✓ {label}\n      error: {e}"),
        }
    }

    // ── 3. Practical: validate user input before submitting ────────────────
    println!("\n── Practical usage ──");

    let user_script = r#"
        const params = runtime.std.getParams();
        if (params.url) {
            const resp = runtime.std.fetch(params.url);
            runtime.std.console.log(resp);
        }
    "#;

    match check_js_syntax(user_script) {
        Ok(()) => println!("  User script is syntactically valid — safe to submit."),
        Err(e) => println!("  User script has syntax error: {e}"),
    }

    let bad_user_script = r#"
        const params = runtime.std.getParams();
        if (params.url) {
            const resp = runtime.std.fetch(params.url
            runtime.std.console.log(resp);
        }
    "#;

    match check_js_syntax(bad_user_script) {
        Ok(()) => println!("  Bad script passed — unexpected!"),
        Err(e) => println!("  Bad script rejected as expected:\n      {e}"),
    }
}
