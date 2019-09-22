/// https://actix.rs/docs/autoreload/
/// cargo install systemfd cargo-watch
/// systemfd --no-pid -s http::3000 -- cargo watch -x run

/// https://github.com/actix/examples/tree/master/diesel
/// https://github.com/actix/examples/tree/master/juniper
///
extern crate actix_web;
extern crate listenfd;
extern crate std;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use listenfd::ListenFd;
use std::sync::Arc;

#[macro_use]
extern crate juniper;
use futures::future::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

mod gql_schema;
mod schema;
use crate::gql_schema::{create_schema, Context, Schema};

#[macro_use]
extern crate diesel;

mod db;
use db::MyPool;
use diesel::r2d2::ConnectionManager;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;

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
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
    pool: web::Data<MyPool<SqliteConnection>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        let res = data.execute(&st, &Context { pool: pool });
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .map_err(Error::from)
    .and_then(|user| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user))
    })
}

fn main() -> std::io::Result<()> {
    // print!("http://127.0.0.1:3000/graphqli",);

    std::env::set_var("RUST_LOG", "actix_web=debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    info!("STARTING SERVER");

    env_logger::init();

    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MyPool::<SqliteConnection>::builder()
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .unwrap();
    // let schema = std::sync::Arc::new(create_schema());
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

    server = if let Some(socket) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(socket).unwrap()
    } else {
        server.bind("localhost:3000").unwrap()
    };

    println!("Started http server: http://127.0.0.1:3000");

    server.run()
}
