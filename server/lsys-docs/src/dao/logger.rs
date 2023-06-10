use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LogDocInfo {
    pub action: &'static str,
    pub name: String,
    pub url: String,
    pub max_try: u8,
}

impl ChangeLogData for LogDocInfo {
    fn log_type<'t>() -> &'t str {
        "git-doc"
    }
    fn message(&self) -> String {
        format!("{} git doc", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogDocTag {
    pub action: &'static str,
    pub doc_git_id: u32,
    pub tag: String,
    pub build_version: String,
    pub clear_rule: String,
}

impl ChangeLogData for LogDocTag {
    fn log_type<'t>() -> &'t str {
        "git-add-tag"
    }
    fn message(&self) -> String {
        format!("clone git {} tag {}", self.action, self.tag)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogDocMenu {
    pub action: &'static str,
    pub doc_tag_id: u64,
    pub menu_path: String,
    pub menu_check_host: String,
}

impl ChangeLogData for LogDocMenu {
    fn log_type<'t>() -> &'t str {
        "git-doc-menu"
    }
    fn message(&self) -> String {
        format!("{} git doc", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogDocClone {
    pub action: &'static str,

    pub doc_tag_id: u64,

    pub host: String,
}

impl ChangeLogData for LogDocClone {
    fn log_type<'t>() -> &'t str {
        "git-clone"
    }
    fn message(&self) -> String {
        format!("clone git tag {}", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
