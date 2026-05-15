use anyhow::Context;
// use diesel::result::DatabaseErrorKind;
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{
        AsyncDieselConnectionManager,
        deadpool::{Object, Pool},
    },
};

use crate::config::DbConfig;

pub struct PostgresStore {
    pool: Pool<AsyncPgConnection>,
}

// impl From<diesel::result::Error> for anyhow::Error {
//     fn from(e: diesel::result::Error) -> Self {
//         use diesel::result::Error::*;
//         match e {
//             DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
//                 anyhow::anyhow!("Unique constraint violation: {}", info.message())
//             }
//             _ => anyhow::anyhow!(e),
//         }
//     }
// }

impl PostgresStore {
    pub fn new(config: &DbConfig) -> anyhow::Result<Self> {
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.url());
        let pool = Pool::builder(manager)
            .max_size(config.max_pool_size())
            .build()
            .context("Failed to build connection pool")?;
        Ok(Self { pool })
    }

    pub(crate) async fn conn(&self) -> anyhow::Result<Object<AsyncPgConnection>> {
        self.pool
            .get()
            .await
            .context("Failed to get connection from pool")
    }
}
