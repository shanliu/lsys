use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LogDocInfo<'t> {
    pub action: &'t str,
    pub name: &'t str,
    pub url: &'t str,
    pub max_try: u8,
    pub user_id: u64,
}

impl ChangeLogData for LogDocInfo<'_> {
    fn log_type() -> &'static str {
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
pub(crate) struct LogDocTag<'t> {
    pub action: &'t str,
    pub doc_git_id: u32,
    pub tag: &'t str,
    pub build_version: &'t str,
    pub clear_rule: &'t str,
    pub user_id: u64,
}

impl ChangeLogData for LogDocTag<'_> {
    fn log_type() -> &'static str {
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
pub(crate) struct LogDocMenu<'t> {
    pub action: &'t str,
    pub doc_tag_id: u64,
    pub menu_path: &'t str,
    pub menu_check_host: &'t str,
    pub user_id: u64,
}

impl ChangeLogData for LogDocMenu<'_> {
    fn log_type() -> &'static str {
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
pub(crate) struct LogDocClone<'t> {
    pub action: &'t str,
    pub doc_tag_id: u64,
    pub host: &'t str,
    pub user_id: u64,
}

impl ChangeLogData for LogDocClone<'_> {
    fn log_type() -> &'static str {
        "git-clone"
    }
    fn message(&self) -> String {
        format!("clone git tag {}", self.action)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
