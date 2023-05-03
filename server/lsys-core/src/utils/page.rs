pub struct PageParam {
    pub offset: u64,
    pub limit: u64,
}
impl PageParam {
    pub fn new(offset: u64, limit: u64) -> Self {
        Self { offset, limit }
    }
    pub fn page(page: u64, limit: u64) -> Self {
        let offset = if page > 0 { (page - 1) * limit } else { 0 };
        Self::new(offset, limit)
    }
}

pub enum LimitParam {
    Next {
        eq_pos: bool,
        pos: Option<u64>,
        limit: u64,
        more: bool,
    },
    Prev {
        eq_pos: bool,
        pos: Option<u64>,
        limit: u64,
        more: bool,
    },
}
impl LimitParam {
    pub fn new(pos: Option<u64>, eq_pos: bool, limit: u64, next: bool, more: bool) -> Self {
        if next {
            Self::Next {
                pos,
                limit,
                more,
                eq_pos,
            }
        } else {
            Self::Prev {
                pos,
                limit,
                more,
                eq_pos,
            }
        }
    }
    pub fn where_sql(&self, field: &str) -> String {
        match self {
            LimitParam::Next {
                pos,
                limit: _,
                more: _,
                eq_pos,
            } => match pos {
                Some(p) => format!(" and {} {} {}", field, if *eq_pos { ">=" } else { ">" }, p),
                None => "".to_string(),
            },
            LimitParam::Prev {
                pos,
                limit: _,
                more: _,
                eq_pos,
            } => match pos {
                Some(p) => format!(" and {} {} {}", field, if *eq_pos { "<=" } else { "<" }, p),
                None => "".to_string(),
            },
        }
    }
    pub fn order_sql(&self, field: &str) -> String {
        match self {
            LimitParam::Next {
                pos: _,
                limit: _,
                more: _,
                eq_pos: _,
            } => {
                format!(" {} asc", field)
            }
            LimitParam::Prev {
                pos: _,
                limit: _,
                more: _,
                eq_pos: _,
            } => {
                format!(" {} desc", field)
            }
        }
    }
    pub fn limit_sql(&self) -> String {
        format!("limit {}", self.limit() + if self.more() { 1 } else { 0 })
    }
    pub fn limit(&self) -> u64 {
        match self {
            LimitParam::Next {
                pos: _,
                limit,
                more: _,
                eq_pos: _,
            } => *limit,
            LimitParam::Prev {
                pos: _,
                limit,
                more: _,
                eq_pos: _,
            } => *limit,
        }
    }
    pub fn pos(&self) -> Option<u64> {
        match self {
            LimitParam::Next {
                pos,
                limit: _,
                more: _,
                eq_pos: _,
            } => *pos,
            LimitParam::Prev {
                pos,
                limit: _,
                more: _,
                eq_pos: _,
            } => *pos,
        }
    }
    pub fn eq_pos(&self) -> bool {
        match self {
            LimitParam::Next {
                pos: _,
                limit: _,
                more: _,
                eq_pos,
            } => *eq_pos,
            LimitParam::Prev {
                pos: _,
                limit: _,
                more: _,
                eq_pos,
            } => *eq_pos,
        }
    }
    pub fn more(&self) -> bool {
        match self {
            LimitParam::Next {
                pos: _,
                limit: _,
                more,
                eq_pos: _,
            } => *more,
            LimitParam::Prev {
                pos: _,
                limit: _,
                more,
                eq_pos: _,
            } => *more,
        }
    }
    pub fn tidy<T>(&self, res: &mut Vec<T>) -> Option<T> {
        if let Self::Next { .. } = self {
            res.reverse();
        };
        if self.more() && res.len() > self.limit() as usize {
            match self {
                LimitParam::Next { .. } => {
                    if !res.is_empty() {
                        Some(res.remove(0))
                    } else {
                        None
                    }
                }
                LimitParam::Prev { .. } => res.pop(),
            }
        } else {
            None
        }
    }
}
