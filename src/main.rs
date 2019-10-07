/// https://actix.rs/docs/autoreload/
/// cargo install systemfd cargo-watch
/// systemfd --no-pid -s http::3000 -- cargo watch -x run

/// https://github.com/actix/examples/tree/master/diesel
/// https://github.com/actix/examples/tree/master/juniper
///
extern crate actix_web;
extern crate listenfd;
extern crate std;

use actix_web::{middleware, web, App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer};
use listenfd::ListenFd;
use std::sync::Arc;

#[macro_use]
extern crate juniper;
use futures::future::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

mod graphql_schema;
use crate::graphql_schema::{create_schema, Context, Schema};

extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;
use r2d2_sqlite::SqliteConnectionManager;

mod models;

mod db;
use db::{db_schema, MyPool};

use dotenv::dotenv;

use std::env;

use failure;

use std::time;

type Result<T> = std::result::Result<T, failure::Error>;

extern crate env_logger;
#[macro_use]
extern crate log;

fn graphiql() -> HttpResponse {
    let html = graphiql_source("http://127.0.0.1:3000/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn graphql(
    req: HttpRequest,
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
    pool: web::Data<MyPool>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let jwt = match req.cookie("t") {
        Some(cookie) => Some(cookie.value().to_owned()),
        None => None,
    };
    web::block(move || {
        let now = time::Instant::now();
        let res = data.execute(
            &st,
            &Context {
                pool: pool,
                jwt: jwt,
            },
        );
        println!("time taken {:?}", now.elapsed());
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .map_err(Error::from)
    .and_then(|user| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user))
    })
}

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    dotenv().ok();
    info!("STARTING SERVER");
    let database_url = env::var("DATABASE_URL")?;
    let manager = SqliteConnectionManager::file(database_url);

    let pool = r2d2::Pool::new(manager)?;

    let connection = pool.get()?;

    connection.execute_batch(&db_schema())?;
    let mut insert_stmt = connection
        .prepare("INSERT INTO posts (content, author_id, parent_id) VALUES (?1, ?2, ?3)")?;
    insert_stmt.execute(&["1", "0", "1"])?;
    insert_stmt.execute(&["1.1", "1", "1"])?;
    insert_stmt.execute(&["1.2", "1", "1"])?;
    insert_stmt.execute(&["1.3", "1", "1"])?;
    insert_stmt.execute(&["1.2.1", "3", "1"])?;
    insert_stmt.execute(&["1.2.2", "3", "1"])?;

    let schema = Arc::new(create_schema());

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/graphql").route(web::post().to_async(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .workers(10);

    server = if let Some(socket) = listenfd.take_tcp_listener(0)? {
        server.listen(socket)?
    } else {
        server.bind("localhost:3000")?
    };

    println!("graphiql started at http://127.0.0.1:3000/graphiql",);
    println!("Started http server: http://127.0.0.1:3000");
    Ok(server.run()?)
}
