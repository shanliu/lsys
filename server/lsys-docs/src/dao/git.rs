use git2::Oid;
use git2::Repository;
use std::fs::read_dir;
use std::fs::remove_dir;
use std::fs::remove_dir_all;
use std::fs::remove_file;
use std::path::Path;
use std::str;
use tracing::error;
use tracing::warn;
use walkdir::WalkDir;

pub fn git_download(git_url: &str, save_dir: &Path, max_num: u8, tag: &str) -> Result<Oid, String> {
    let mut try_num = 0;
    loop {
        if Path::new(save_dir).is_dir() {
            match remove_dir_all(save_dir) {
                Ok(_) => continue,
                Err(err) => {
                    let msg = format!("remove dir {} fail:{}", save_dir.to_string_lossy(), err);
                    error!(msg);
                    break Err(msg);
                }
            }
        };
        let repo = match Repository::clone(git_url, save_dir) {
            Ok(tmp) => tmp,
            Err(err) => {
                try_num += 1;
                error!("git clone {} fail:{}", save_dir.to_string_lossy(), err);
                if try_num > max_num {
                    break Err(format!("git clone fail,max try:{}", max_num));
                }
                continue;
            }
        };
        let obj = match repo.revparse_single(tag) {
            Ok(obj) => obj,
            Err(err) => {
                let msg = format!("git fin tag {} fail:{}", save_dir.to_string_lossy(), err);
                error!(msg);
                break Err(msg);
            }
        };
        match repo.set_head_detached(obj.id()) {
            Ok(_) => {
                break Ok(obj.id());
            }
            Err(err) => {
                let msg = format!("git fin tag {} fail:{}", save_dir.to_string_lossy(), err);
                error!(msg);
                break Err(msg);
            }
        }
    }
}

pub fn git_clear(path: &Path, clear_rule: &Option<Vec<String>>) {
    match remove_dir_all(path.join(".git")) {
        Ok(_) => {}
        Err(err) => {
            warn!("{} clear .git dir fail:{}", path.to_string_lossy(), err);
        }
    }
    if let Some(clear_rule) = clear_rule {
        let regex_array = clear_rule
            .iter()
            .filter_map(|e| regex::Regex::new(e).ok())
            .collect::<Vec<_>>();
        if !regex_array.is_empty() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    for regex in &regex_array {
                        if let Some(p) = path.to_str() {
                            if regex.is_match(p) {
                                if let Err(err) = remove_file(path) {
                                    warn!("delete git file {} error:{}", p, err);
                                }
                                break;
                            }
                        }
                    }
                }
            }
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_dir() {
                    let dir_rs = read_dir(entry.path());
                    if let Ok(mut dir) = dir_rs {
                        if dir.next().is_none() {
                            let _ = remove_dir(entry.path());
                        }
                    }
                }
            }
        }
    }
}
