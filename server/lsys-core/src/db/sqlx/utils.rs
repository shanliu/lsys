use super::value::StoredValue;
use sqlx::{Database, QueryBuilder};

/// 将字段作为 SET 子句（`col = value` 格式）添加到 QueryBuilder
pub(crate) fn push_set_clause_to<DB: Database>(
    fields: &[(String, StoredValue<DB>)],
    qb: &mut QueryBuilder<'_, DB>,
) {
    let mut first = true;
    for (col, value) in fields {
        if !first {
            qb.push(", ");
        }
        first = false;

        qb.push(col.as_str());
        qb.push(" = ");

        match value {
            StoredValue::Bind(b) => {
                b.bind_to(qb);
            }
            StoredValue::Expr(e) => {
                qb.push(e.as_ref());
            }
            StoredValue::Dynamic(f) => {
                f(qb);
            }
        }
    }
}

/// 将字段作为 VALUES 格式（只有 `value`）添加到 QueryBuilder
pub(crate) fn push_values_to<DB: Database>(
    fields: &[(String, StoredValue<DB>)],
    qb: &mut QueryBuilder<'_, DB>,
) {
    let mut first = true;
    for (_, value) in fields {
        if !first {
            qb.push(", ");
        }
        first = false;

        match value {
            StoredValue::Bind(b) => {
                b.bind_to(qb);
            }
            StoredValue::Expr(e) => {
                qb.push(e.as_ref());
            }
            StoredValue::Dynamic(f) => {
                f(qb);
            }
        }
    }
}

/// 处理单个字段值到 QueryBuilder，支持默认值
/// 如果字段在 fields 中存在，将其绑定到 QueryBuilder
/// 如果不存在，推入 default_value
pub(crate) fn push_field_value_or_default<'args, DB: Database>(
    fields: &[(String, StoredValue<DB>)],
    col: &str,
    qb: &mut QueryBuilder<'args, DB>,
    default_value: &str,
) {
    if let Some((_, stored)) = fields.iter().find(|(c, _)| c == col) {
        match stored {
            StoredValue::Bind(b) => {
                b.bind_to(qb);
            }
            StoredValue::Expr(e) => {
                qb.push(e.as_ref());
            }
            StoredValue::Dynamic(f) => {
                f(qb);
            }
        }
    } else {
        qb.push(default_value);
    }
}
