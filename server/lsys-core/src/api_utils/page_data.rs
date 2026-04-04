use serde::Serialize;

use crate::db::{CursorPageData, TotalRow};

/// TotalRow 的 JSON 响应对象。
///
/// - `exact`：精确数量（Over 时为 null）
/// - `over`：超过阈值时的阈值（Exact 时为 null）
#[derive(Debug, Clone, Serialize)]
pub struct PageTotalRowValue {
    pub exact: Option<u64>,
    pub over: Option<u64>,
}

impl From<TotalRow> for PageTotalRowValue {
    fn from(value: TotalRow) -> Self {
        match value {
            TotalRow::Exact(n) => Self {
                exact: Some(n),
                over: None,
            },
            TotalRow::Over(n) => Self {
                exact: None,
                over: Some(n),
            },
        }
    }
}

impl From<&TotalRow> for PageTotalRowValue {
    fn from(value: &TotalRow) -> Self {
        match value {
            TotalRow::Exact(n) => Self {
                exact: Some(*n),
                over: None,
            },
            TotalRow::Over(n) => Self {
                exact: None,
                over: Some(*n),
            },
        }
    }
}

/// CursorPageData<u64> 的 JSON 响应对象。
///
/// - `next`：下一页游标（无时为 null）
/// - `prev`：上一页游标（无时为 null）
#[derive(Debug, Clone, Serialize)]
pub struct PageCursorValue {
    pub next: Option<u64>,
    pub prev: Option<u64>,
}

impl From<CursorPageData<u64>> for PageCursorValue {
    fn from(value: CursorPageData<u64>) -> Self {
        Self {
            next: value.next_cursor,
            prev: value.prev_cursor,
        }
    }
}

impl From<&CursorPageData<u64>> for PageCursorValue {
    fn from(value: &CursorPageData<u64>) -> Self {
        Self {
            next: value.next_cursor,
            prev: value.prev_cursor,
        }
    }
}
