//! Utility functions – lightweight helpers that do **not** require a full
//! [`JsEngine`](crate::JsEngine) or tokio runtime.

/// Information about a syntax error found in a JS script.
///
/// The `message` field contains the QuickJS error string, with line numbers
/// adjusted to match the original source (1-based).
#[derive(Debug, Clone)]
pub struct JsSyntaxError {
    /// Human-readable error description (typically includes line number).
    pub message: String,
}

impl ::std::fmt::Display for JsSyntaxError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ::std::error::Error for JsSyntaxError {}

/// Check whether a JS script is syntactically valid.
///
/// Creates a lightweight, throw-away QuickJS context, wraps the source in a
/// function body to trigger **parsing without execution**, and returns any
/// syntax error found.
///
/// This function is **synchronous** and does **not** require a [`JsEngine`],
/// a tokio runtime, or any prior setup – it is intended as a quick pre-flight
/// check before submitting code to a full runtime.
///
/// # Examples
///
/// ```rust
/// use lsys_lib_jsrun::check_js_syntax;
///
/// // Valid script
/// assert!(check_js_syntax("let x = 1 + 2;").is_ok());
///
/// // Syntax error
/// let err = check_js_syntax("let x = ;").unwrap_err();
/// assert!(err.message.to_lowercase().contains("syntax"));
/// ```
pub fn check_js_syntax(source: &str) -> Result<(), JsSyntaxError> {
    use rquickjs::CatchResultExt;

    let rt = rquickjs::Runtime::new().map_err(|e| JsSyntaxError {
        message: format!("Failed to create QuickJS runtime: {e}"),
    })?;
    let ctx = rquickjs::Context::full(&rt).map_err(|e| JsSyntaxError {
        message: format!("Failed to create QuickJS context: {e}"),
    })?;

    ctx.with(|ctx| {
        // Wrap the source in a function body so that QuickJS **parses** the
        // code without executing it.  This is the standard technique for
        // syntax validation (the same approach Node.js uses for `require`).
        let wrapper = format!("(function(){{\n{}\n}});", source);

        let result: Result<rquickjs::Value<'_>, _> = ctx.eval(wrapper).catch(&ctx);

        match result {
            Ok(_) => Ok(()),
            Err(err) => {
                let msg = match err {
                    rquickjs::CaughtError::Error(e) => format!("{e}"),
                    rquickjs::CaughtError::Exception(ex) => {
                        // QuickJS reports the line inside the wrapper;
                        // adjust by −1 to match the user's original source.
                        adjust_syntax_error_line(&format!("{ex}"), 1)
                    }
                    rquickjs::CaughtError::Value(v) => format!("{v:?}"),
                };
                Err(JsSyntaxError { message: msg })
            }
        }
    })
}

/// Adjust `"line N"` references in a QuickJS error message by subtracting
/// `offset`, so that the reported line matches the user's original source.
fn adjust_syntax_error_line(msg: &str, offset: usize) -> String {
    // QuickJS error format: "SyntaxError: …, line 5"
    if let Some(idx) = msg.rfind("line ") {
        let prefix = &msg[..idx];
        let after = &msg[idx + 5..];

        // Find the end of the number
        let num_end = after
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after.len());

        if let Ok(n) = after[..num_end].parse::<usize>() {
            let adjusted = n.saturating_sub(offset).max(1);
            return format!("{}line {}{}", prefix, adjusted, &after[num_end..]);
        }
    }
    msg.to_string()
}
