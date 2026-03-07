//! `core.File` – Sandboxed file I/O.
//!
//! Files are created inside a fixed working directory.
//! File names must NOT contain `/` or `\` (path separators) to prevent traversal.
//! The runtime automatically closes all open files on exit.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rquickjs::{class::Trace, function::Opt, Ctx, JsLifetime, Result as JsResult};

use crate::runtime::FileLocalSyncHandler;

/// Type alias for a file handle wrapped in Arc<Mutex<Option<File>>>.
type FileHandle = Arc<Mutex<Option<File>>>;

/// Type alias for the open files tracker.
type OpenFiles = Arc<Mutex<Vec<FileHandle>>>;

/// Validate that a filename is safe (no path separators, no `..`).
fn validate_filename(name: &str) -> std::result::Result<(), String> {
    if name.is_empty() {
        return Err("File name cannot be empty".into());
    }
    if name.contains('/') || name.contains('\\') {
        return Err(format!(
            "File name '{}' must not contain path separators",
            name
        ));
    }
    if name == "." || name == ".." || name.contains("..") {
        return Err(format!("File name '{}' is not allowed", name));
    }
    Ok(())
}

/// Shared tracker for open files so we can close them all on runtime shutdown.
#[derive(Clone)]
pub struct FileTracker {
    work_dir: PathBuf,
    open_files: OpenFiles,
    file_sync_handler: Option<FileLocalSyncHandler>,
    namespace: Option<String>,
    tokio_handle: tokio::runtime::Handle,
}

// SAFETY: FileTracker contains no JS values – it is entirely `'static`.
unsafe impl<'js> rquickjs::JsLifetime<'js> for FileTracker {
    type Changed<'to> = FileTracker;
}

impl FileTracker {
    pub fn new(
        work_dir: PathBuf,
        file_sync_handler: Option<FileLocalSyncHandler>,
        namespace: Option<String>,
        tokio_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            work_dir,
            open_files: Arc::new(Mutex::new(Vec::new())),
            file_sync_handler,
            namespace,
            tokio_handle,
        }
    }

    /// Close all tracked files (called on runtime teardown).
    pub fn close_all(&self) {
        if let Ok(mut files) = self.open_files.lock() {
            for f in files.drain(..) {
                if let Ok(mut guard) = f.lock() {
                    let _ = guard.take(); // drops the File, closing it
                }
            }
        }
    }

    fn register(&self, handle: FileHandle) {
        if let Ok(mut files) = self.open_files.lock() {
            files.push(handle);
        }
    }

    fn full_path(&self, name: &str) -> std::result::Result<PathBuf, String> {
        validate_filename(name)?;
        Ok(self.work_dir.join(name))
    }
}

/// JS-visible `File` class.
#[derive(Trace, JsLifetime)]
#[rquickjs::class(rename = "File")]
pub struct JsFile {
    #[qjs(skip_trace)]
    name: String,
    #[qjs(skip_trace)]
    handle: FileHandle,
    #[qjs(skip_trace)]
    tracker: FileTracker,
}

#[rquickjs::methods]
impl JsFile {
    /// `new File(name)` – open or create a file in the working directory.
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>, name: String) -> JsResult<Self> {
        // Retrieve the FileTracker from context userdata
        let tracker_guard = ctx.userdata::<FileTracker>().ok_or_else(|| {
            rquickjs::Error::new_from_js_message("ctx", "userdata", "FileTracker not found")
        })?;
        let tracker: FileTracker = (*tracker_guard).clone();

        let path = tracker
            .full_path(&name)
            .map_err(|e| rquickjs::Error::new_from_js_message("string", "path", e))?;

        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|e| rquickjs::Error::new_from_js_message("path", "file", e.to_string()))?;

        let handle = Arc::new(Mutex::new(Some(file)));
        tracker.register(handle.clone());

        Ok(Self {
            name,
            handle,
            tracker,
        })
    }

    /// `file.write(data)` – write a string to the file, returns bytes written.
    pub fn write(&self, data: String) -> JsResult<usize> {
        let mut guard = self
            .handle
            .lock()
            .map_err(|_| rquickjs::Error::new_from_js_message("lock", "guard", "mutex poisoned"))?;
        let f = guard.as_mut().ok_or_else(|| {
            rquickjs::Error::new_from_js_message("file", "handle", "file is closed")
        })?;
        f.write(data.as_bytes())
            .map_err(|e| rquickjs::Error::new_from_js_message("write", "usize", e.to_string()))
    }

    /// `file.seek(offset)` – seek to an absolute position.
    pub fn seek(&self, offset: u64) -> JsResult<u64> {
        let mut guard = self
            .handle
            .lock()
            .map_err(|_| rquickjs::Error::new_from_js_message("lock", "guard", "mutex poisoned"))?;
        let f = guard.as_mut().ok_or_else(|| {
            rquickjs::Error::new_from_js_message("file", "handle", "file is closed")
        })?;
        f.seek(SeekFrom::Start(offset))
            .map_err(|e| rquickjs::Error::new_from_js_message("seek", "u64", e.to_string()))
    }

    /// `file.tell()` – return current file pointer position.
    pub fn tell(&self) -> JsResult<u64> {
        let mut guard = self
            .handle
            .lock()
            .map_err(|_| rquickjs::Error::new_from_js_message("lock", "guard", "mutex poisoned"))?;
        let f = guard.as_mut().ok_or_else(|| {
            rquickjs::Error::new_from_js_message("file", "handle", "file is closed")
        })?;
        f.stream_position()
            .map_err(|e| rquickjs::Error::new_from_js_message("tell", "u64", e.to_string()))
    }

    /// `file.read(size?)` – read bytes. If size is omitted, read to end.
    pub fn read(&self, size: Opt<usize>) -> JsResult<String> {
        let mut guard = self
            .handle
            .lock()
            .map_err(|_| rquickjs::Error::new_from_js_message("lock", "guard", "mutex poisoned"))?;
        let f = guard.as_mut().ok_or_else(|| {
            rquickjs::Error::new_from_js_message("file", "handle", "file is closed")
        })?;

        let buf = match size.0 {
            Some(n) => {
                let mut b = vec![0u8; n];
                let read = f.read(&mut b).map_err(|e| {
                    rquickjs::Error::new_from_js_message("read", "bytes", e.to_string())
                })?;
                b.truncate(read);
                b
            }
            None => {
                let mut b = Vec::new();
                f.read_to_end(&mut b).map_err(|e| {
                    rquickjs::Error::new_from_js_message("read", "bytes", e.to_string())
                })?;
                b
            }
        };

        String::from_utf8(buf)
            .map_err(|e| rquickjs::Error::new_from_js_message("bytes", "string", e.to_string()))
    }

    /// `file.close()` – explicitly close the file.
    pub fn close(&self) -> JsResult<()> {
        let mut guard = self
            .handle
            .lock()
            .map_err(|_| rquickjs::Error::new_from_js_message("lock", "guard", "mutex poisoned"))?;
        let _ = guard.take();
        Ok(())
    }

    /// `file.local_sync()` – synchronise the file via the host-provided `FileLocalSyncHandler`.
    ///
    /// Calls the async handler configured in `RuntimeConfig::file_sync_handler`
    /// with the runtime namespace, the file's full path, and the work directory.
    /// Returns the handler's result as a JSON string.
    ///
    /// Throws if no `file_sync_handler` has been configured.
    pub fn local_sync(&self) -> JsResult<String> {
        let handler = self.tracker.file_sync_handler.as_ref().ok_or_else(|| {
            rquickjs::Error::new_from_js_message(
                "file",
                "local_sync",
                "file_sync_handler is not configured",
            )
        })?;

        let full_path = self
            .tracker
            .full_path(&self.name)
            .map_err(|e| rquickjs::Error::new_from_js_message("string", "path", e))?;
        let work_dir = self.tracker.work_dir.clone();
        let namespace = self.tracker.namespace.clone();
        let handler = handler.clone();
        let handle = &self.tracker.tokio_handle;

        let result = super::block_on_async(handle, async move {
            handler(namespace, full_path, work_dir).await
        });

        match result {
            Ok(val) => serde_json::to_string(&val).map_err(|e| {
                rquickjs::Error::new_from_js_message("json", "string", e.to_string())
            }),
            Err(e) => Err(rquickjs::Error::new_from_js_message("file", "local_sync", e)),
        }
    }

    /// `file.rename(newName)` – rename the file.
    pub fn rename(&mut self, new_name: String) -> JsResult<()> {
        let old_path = self
            .tracker
            .full_path(&self.name)
            .map_err(|e| rquickjs::Error::new_from_js_message("string", "path", e))?;
        let new_path = self
            .tracker
            .full_path(&new_name)
            .map_err(|e| rquickjs::Error::new_from_js_message("string", "path", e))?;

        // close the handle first
        {
            let mut guard = self.handle.lock().map_err(|_| {
                rquickjs::Error::new_from_js_message("lock", "guard", "mutex poisoned")
            })?;
            let _ = guard.take();
        }

        std::fs::rename(&old_path, &new_path)
            .map_err(|e| rquickjs::Error::new_from_js_message("rename", "void", e.to_string()))?;

        // reopen with new path
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&new_path)
            .map_err(|e| rquickjs::Error::new_from_js_message("path", "file", e.to_string()))?;

        {
            let mut guard = self.handle.lock().map_err(|_| {
                rquickjs::Error::new_from_js_message("lock", "guard", "mutex poisoned")
            })?;
            *guard = Some(file);
        }

        self.name = new_name;
        Ok(())
    }

    /// `File.exists(name)` – static: check if a file exists in the work directory.
    #[qjs(static)]
    pub fn exists(ctx: Ctx<'_>, name: String) -> JsResult<bool> {
        let tracker_guard = ctx.userdata::<FileTracker>().ok_or_else(|| {
            rquickjs::Error::new_from_js_message("ctx", "userdata", "FileTracker not found")
        })?;
        let tracker: FileTracker = (*tracker_guard).clone();
        let path = tracker
            .full_path(&name)
            .map_err(|e| rquickjs::Error::new_from_js_message("string", "path", e))?;
        Ok(path.exists())
    }

    /// `File.getsize(name)` – static: return file size in bytes.
    #[qjs(static)]
    pub fn getsize(ctx: Ctx<'_>, name: String) -> JsResult<f64> {
        let tracker_guard = ctx.userdata::<FileTracker>().ok_or_else(|| {
            rquickjs::Error::new_from_js_message("ctx", "userdata", "FileTracker not found")
        })?;
        let tracker: FileTracker = (*tracker_guard).clone();
        let path = tracker
            .full_path(&name)
            .map_err(|e| rquickjs::Error::new_from_js_message("string", "path", e))?;
        let meta = std::fs::metadata(&path)
            .map_err(|e| rquickjs::Error::new_from_js_message("path", "metadata", e.to_string()))?;
        Ok(meta.len() as f64)
    }

    /// `File.remove(name)` – static: delete a file from the work directory.
    #[qjs(static)]
    pub fn remove(ctx: Ctx<'_>, name: String) -> JsResult<()> {
        let tracker_guard = ctx.userdata::<FileTracker>().ok_or_else(|| {
            rquickjs::Error::new_from_js_message("ctx", "userdata", "FileTracker not found")
        })?;
        let tracker: FileTracker = (*tracker_guard).clone();
        let path = tracker
            .full_path(&name)
            .map_err(|e| rquickjs::Error::new_from_js_message("string", "path", e))?;
        std::fs::remove_file(&path)
            .map_err(|e| rquickjs::Error::new_from_js_message("path", "void", e.to_string()))
    }
}
