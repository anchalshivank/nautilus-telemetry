use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, Pool, Postgres};

pub async fn get_pool(database_url: String, connections: u32) -> Result<Pool<Postgres>, Error> {
    PgPoolOptions::new()
        .max_connections(connections)
        .connect(&database_url)
        .await
}
