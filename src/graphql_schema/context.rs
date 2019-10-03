use crate::db::MyPool;

pub struct Context {
    pub pool: actix_web::web::Data<MyPool>,
    pub jwt: Option<String>,
}

impl juniper::Context for Context {}
