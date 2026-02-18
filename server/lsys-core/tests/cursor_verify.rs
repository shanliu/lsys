#![cfg(feature = "db")]

use lsys_core::db::{
    CursorConfig, CursorPageDir, CursorPageParam, CursorPageSort, CursorValue, SqlQuote,
};
use std::fmt::Display;

#[derive(Debug, Clone)]
struct UserCursor {
    id: u64,
    name: String,
}

impl CursorValue for UserCursor {
    fn key_value(&self) -> Box<dyn Display> {
        Box::new(self.id)
    }
    fn extra_values(&self) -> Vec<(String, Box<dyn Display>)> {
        vec![("name".to_string(), Box::new(self.name.sql_quote()))]
    }
}

fn print_sql_scenario(
    title: &str,
    where_sql: String,
    order_sql: String,
    limit_sql: String,
    extra_where: Option<&str>,
) {
    println!("\n=== {} ===", title);
    let final_where = if let Some(extra) = extra_where {
        if where_sql.is_empty() {
            format!("WHERE {}", extra)
        } else {
            format!("WHERE {} AND {}", extra, where_sql)
        }
    } else if where_sql.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", where_sql)
    };

    println!(
        "SQL: SELECT * FROM table {} {} {}",
        final_where, order_sql, limit_sql
    );
}

#[test]
fn verify_cursor_sql_generation() {
    println!("---------------------------------------------------------------");
    println!("VERIFICATION 1: Single ID Sorting (Primary Key Only)");
    println!("---------------------------------------------------------------");

    // Case 1: First Page (No Cursor), Next, ASC
    let param: CursorPageParam<u64> = CursorPageParam {
        dir: CursorPageDir::Next,
        cursor: None,
        config: CursorConfig::primary(CursorPageSort::Asc),
        limit: lsys_core::db::CursorLimit::Limit {
            limit: 10,
            more: true,
        },
    };

    let query = param.page_query("id");
    print_sql_scenario(
        "1.1 First Page (Asc, Next)",
        query.where_sql(),
        query.order_by_sql(),
        query.limit_sql().unwrap_or_default(),
        Some("status = 1"),
    );

    // Case 2: Next Page (With Cursor = 100), ASC
    let param: CursorPageParam<u64> = CursorPageParam {
        dir: CursorPageDir::Next,
        cursor: Some(100),
        config: CursorConfig::primary(CursorPageSort::Asc),
        limit: lsys_core::db::CursorLimit::Limit {
            limit: 10,
            more: true,
        },
    };
    let query = param.page_query("id");
    print_sql_scenario(
        "1.2 Next Page (Cursor=100, Asc, Next)",
        query.where_sql(),
        query.order_by_sql(),
        query.limit_sql().unwrap_or_default(),
        None,
    );

    // Case 3: Prev Page (With Cursor = 100), ASC
    let param: CursorPageParam<u64> = CursorPageParam {
        dir: CursorPageDir::Prev,
        cursor: Some(100),
        config: CursorConfig::primary(CursorPageSort::Asc),
        limit: lsys_core::db::CursorLimit::Limit {
            limit: 10,
            more: true,
        },
    };
    let query = param.page_query("id");
    print_sql_scenario(
        "1.3 Prev Page (Cursor=100, Asc, Prev)",
        query.where_sql(),
        query.order_by_sql(),
        query.limit_sql().unwrap_or_default(),
        Some("group_id = 5"),
    );

    println!("\n---------------------------------------------------------------");
    println!("VERIFICATION 2: Multi-Sort (Name DESC, ID ASC)");
    println!("---------------------------------------------------------------");

    let cursor_data = UserCursor {
        id: 50,
        name: "Alice".to_string(),
    };

    // Case 4: First Page (No Cursor)
    let param: CursorPageParam<UserCursor> = CursorPageParam {
        dir: CursorPageDir::Next,
        cursor: None,
        config: CursorConfig::new(
            CursorPageSort::Desc,
            vec![("name".to_string(), CursorPageSort::Desc)],
        ),
        limit: lsys_core::db::CursorLimit::Limit {
            limit: 10,
            more: true,
        },
    };
    let query = param.page_query("id");
    print_sql_scenario(
        "2.1 First Page (Multi-Sort)",
        query.where_sql(),
        query.order_by_sql(),
        query.limit_sql().unwrap_or_default(),
        None,
    );

    // Case 5: Next Page with Cursor
    let param: CursorPageParam<UserCursor> = CursorPageParam {
        dir: CursorPageDir::Next,
        cursor: Some(cursor_data.clone()),
        config: CursorConfig::new(
            CursorPageSort::Desc,
            vec![("name".to_string(), CursorPageSort::Desc)],
        ),
        limit: lsys_core::db::CursorLimit::Limit {
            limit: 10,
            more: true,
        },
    };
    let query = param.page_query("id");
    print_sql_scenario(
        "2.2 Next Page (Cursor='Alice', 50)",
        query.where_sql(),
        query.order_by_sql(),
        query.limit_sql().unwrap_or_default(),
        Some("active = true"),
    );

    // Case 6: Prev Page with Cursor
    let param: CursorPageParam<UserCursor> = CursorPageParam {
        dir: CursorPageDir::Prev,
        cursor: Some(cursor_data.clone()),
        limit: lsys_core::db::CursorLimit::Limit {
            limit: 10,
            more: true,
        },
        config: CursorConfig::new(
            CursorPageSort::Desc,
            vec![("name".to_string(), CursorPageSort::Desc)],
        ),
    };
    let query = param.page_query("id");
    print_sql_scenario(
        "2.3 Prev Page (Cursor='Alice', 50)",
        query.where_sql(),
        query.order_by_sql(),
        query.limit_sql().unwrap_or_default(),
        None,
    );
}
