use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type MyPool = Pool<SqliteConnectionManager>;
// infer_schema!("dotenv:DATABASE_URL");

pub type MyPooledConnection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;
