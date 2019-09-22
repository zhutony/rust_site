use diesel::r2d2::{ConnectionManager, Pool};

pub type MyPool<ConnectionType> = Pool<ConnectionManager<ConnectionType>>;
// infer_schema!("dotenv:DATABASE_URL");
