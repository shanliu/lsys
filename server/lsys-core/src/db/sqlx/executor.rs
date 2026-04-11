use futures_util::future::BoxFuture;
use futures_util::stream::BoxStream;
use sqlx::{Database, Describe, Either, Execute, Executor, Pool, error::Error};
use std::fmt::Debug;

pub enum OptionTxExecutor<'c, DB: Database> {
    Pool(&'c Pool<DB>),
    Transaction(&'c mut <DB as Database>::Connection),
}

impl<'c, DB: Database> Debug for OptionTxExecutor<'c, DB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionTxExecutor::Pool(_) => f.write_str("OptionTxExecutor::Pool"),
            OptionTxExecutor::Transaction(_) => f.write_str("OptionTxExecutor::Transaction"),
        }
    }
}

impl<'c, DB: Database> OptionTxExecutor<'c, DB> {
    pub fn new(transaction: Option<&'c mut sqlx::Transaction<'_, DB>>, pool: &'c Pool<DB>) -> Self {
        match transaction {
            Some(t) => OptionTxExecutor::Transaction(&mut **t),
            None => OptionTxExecutor::Pool(pool),
        }
    }
}

impl<'c, DB: Database> Executor<'c> for OptionTxExecutor<'c, DB>
where
    for<'e> &'e Pool<DB>: Executor<'e, Database = DB>,
    for<'e> &'e mut <DB as Database>::Connection: Executor<'e, Database = DB>,
{
    type Database = DB;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxStream<
        'e,
        Result<Either<<DB as sqlx::Database>::QueryResult, <DB as sqlx::Database>::Row>, Error>,
    >
    where
        'c: 'e,
        E: Execute<'q, Self::Database> + 'q,
    {
        match self {
            OptionTxExecutor::Pool(p) => p.fetch_many(query),
            OptionTxExecutor::Transaction(t) => t.fetch_many(query),
        }
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<<DB as sqlx::Database>::Row>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database> + 'q,
    {
        match self {
            OptionTxExecutor::Pool(p) => p.fetch_optional(query),
            OptionTxExecutor::Transaction(t) => t.fetch_optional(query),
        }
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
    ) -> BoxFuture<'e, Result<<Self::Database as sqlx::Database>::Statement<'q>, Error>>
    where
        'c: 'e,
    {
        match self {
            OptionTxExecutor::Pool(p) => p.prepare_with(sql, parameters),
            OptionTxExecutor::Transaction(t) => t.prepare_with(sql, parameters),
        }
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
    where
        'c: 'e,
    {
        match self {
            OptionTxExecutor::Pool(p) => p.describe(sql),
            OptionTxExecutor::Transaction(t) => t.describe(sql),
        }
    }
}
