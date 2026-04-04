//! Batch runner example – demonstrates submitting multiple JS tasks
//! with per-task completion callbacks.

use lsys_lib_jsrun::{EngineConfig, JsEngine, JsTaskRunner, RuntimeConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // ── 1. Create the engine ────────────────────────────────
    let engine = JsEngine::new(EngineConfig {
        max_runtimes: 4,
        ..Default::default()
    })?;

    // ── 2. Create the task runner ─────────────────────────────────
    let runner = std::sync::Arc::new(JsTaskRunner::new(engine, RuntimeConfig::default()));
    tokio::spawn({ let r = runner.clone(); async move { r.run().await; } });
    tokio::spawn({ let r = runner.clone(); async move { r.run_engine_cleanup().await; } });

    // ── 3. Submit tasks with per-task callbacks ──────────────
    println!("Submitting 6 tasks (engine max_runtimes=4, so 2 will queue)...\n");

    let mut handles = Vec::new();
    for i in 1..=6 {
        let code = format!(
            r#"
            var result = {i} * {i};
            runtime.std.console.log("Task {i}: {i} * {i} = " + result);
            result
            "#,
            i = i
        );
        // Each task has its own completion callback
        let h = runner.submit(
            code,
            None,
            Some(move |result: lsys_lib_jsrun::TaskResult| async move {
                println!(
                    "  ✅ [callback] Task #{} completed in {:?} → {:?}",
                    result.task_id, result.elapsed, result.outcome
                );
            }),
        ).await;
        println!("  📤 Submitted task #{}", h.task_id);
        handles.push(h);
    }

    // ── 4. Await a specific task ────────────────────────────
    println!("\nAwaiting task #3 specifically...");
    if let Some(result3) = handles.remove(2).await_result().await {
        println!(
            "  🎯 Task #3 result: {:?} (took {:?})",
            result3.outcome, result3.elapsed
        );
    }

    // ── 5. Continuously add more tasks while others are running ──
    println!("\nAdding 3 more tasks dynamically...");
    for i in 7..=9 {
        let code = format!(
            "runtime.std.console.log('Dynamic task {i}'); {i} * 100",
            i = i
        );
        // These tasks have no callback – just await the handle
        let h = runner.submit_simple(code, None).await;
        println!("  📤 Submitted dynamic task #{}", h.task_id);
        handles.push(h);
    }

    // ── 6. Await all remaining handles ──────────────────────
    println!("\nAwaiting all remaining tasks...");
    for h in handles {
        if let Some(r) = h.await_result().await {
            println!(
                "  📦 Task #{}: {:?} ({:?})",
                r.task_id, r.outcome, r.elapsed
            );
        }
    }

    // ── 7. Shutdown ─────────────────────────────────────────
    println!("\nShutting down runner...");
    runner.shutdown().await;
    println!("✅ All done!");

    Ok(())
}
