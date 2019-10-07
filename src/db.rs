use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type MyPool = Pool<SqliteConnectionManager>;
// infer_schema!("dotenv:DATABASE_URL");

pub type MyPooledConnection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub fn db_schema() -> String {
    "
                BEGIN TRANSACTION;
                DROP TABLE IF EXISTS posts;
                DROP TABLE IF EXISTS users;
                
                CREATE TABLE IF NOT EXISTS posts( 
                    id INTEGER NOT NULL PRIMARY KEY, 
                    content TEXT, 
                    author_id INTEGER NOT NULL REFERENCES users (id) NOT NULL,
                    parent_id INTEGER REFERENCES posts (id) NOT NULL
                );

                CREATE TABLE IF NOT EXISTS users ( 
                    id INTEGER NOT NULL PRIMARY KEY, 
                    username VARCHAR (255) UNIQUE, 
                    email VARCHAR (255) UNIQUE, 
                    hash VARCHAR (255), 
                    firstname VARCHAR (255), 
                    lastname VARCHAR (255)
                );
                COMMIT;
            "
    .to_owned()
}
