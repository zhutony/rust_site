use juniper::{FieldResult, RootNode};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::db::MyPool;

use crate::schema::posts;

#[macro_use]
use serde_derive;

use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};

pub struct Context {
    // pub pool: MyPool<SqliteConnection>,
    pub pool: actix_web::web::Data<MyPool<SqliteConnection>>,
}

impl juniper::Context for Context {}

#[derive(Queryable, Debug, PartialEq)]
struct Post {
    id: i32,
    content: String,
    parent_id: i32,
}

#[juniper::object(
    Context = Context,
)]
#[graphql(description = "A post")]
impl Post {
    fn id(&self) -> i32 {
        self.id
    }
    fn content(&self) -> &str {
        self.content.as_str()
    }
    fn parent_id(&self) -> i32 {
        self.parent_id
    }
    fn parent(&self, context: &Context) -> FieldResult<Post> {
        let temp_parent_id = self.parent_id.clone();
        if temp_parent_id == 0i32 {
            let result = Post {
                id: 0i32,
                content: "ROOT".to_owned(),
                parent_id: 0i32,
            };
            Ok(result)
        } else {
            let connection = &context.pool.get().unwrap();
            use crate::schema::posts::dsl::*;
            let result = posts
                .filter(crate::schema::posts::dsl::id.eq(temp_parent_id))
                .get_result::<Post>(connection)?;
            Ok(result)
            // let result = Post {
            //     id: 0i32,
            //     content: "NOT ROOT".to_owned(),
            //     parent_id: 0i32,
            // };
            // Ok(result)
        }
    }
    fn children(&self, context: &Context) -> FieldResult<Vec<Post>> {
        let connection = &context.pool.get().unwrap();
        use crate::schema::posts::dsl::*;
        let result = posts
            .filter(parent_id.eq(&self.id))
            .load::<Post>(connection)?;
        Ok(result)
    }
}

#[derive(GraphQLInputObject, Insertable, Debug)]
#[table_name = "posts"]
struct NewPost {
    content: String,
    parent_id: i32,
}

pub struct QueryRoot;

#[juniper::object(
    Context = Context,
)]
impl QueryRoot {
    fn posts(context: &Context) -> FieldResult<Vec<Post>> {
        let connection = &context.pool.get().unwrap();
        use crate::schema::posts::dsl::*;
        let result = posts.limit(100).load::<Post>(connection)?;
        Ok(result)
    }
    fn post(context: &Context, post_id: i32) -> FieldResult<Post> {
        if post_id == 0i32 {
            let result = Post {
                id: 0i32,
                content: "ROOT".to_owned(),
                parent_id: 0i32,
            };
            Ok(result)
        } else {
            let connection = &context.pool.get().unwrap();
            use crate::schema::posts::dsl::*;
            let result = posts
                .filter(crate::schema::posts::dsl::id.eq(post_id))
                .get_result::<Post>(connection)?;
            Ok(result)
        }
    }
}

pub struct MutationRoot;

#[juniper::object(
    Context = Context,
)]
impl MutationRoot {
    fn createPost(context: &Context, new_post: NewPost) -> FieldResult<bool> {
        let connection = &context.pool.get().unwrap();
        use crate::schema::posts::dsl::*;
        let result = diesel::insert_into(posts)
            .values(&new_post)
            .execute(connection)?;
        // let result = diesel::insert_into(posts).values(&new_post).get_result::<Post>(connection)?;
        Ok(true)
    }
    fn deletePost(context: &Context, post_id: i32) -> FieldResult<bool> {
        let connection = &context.pool.get().unwrap();
        use crate::schema::posts::dsl::*;
        let result = diesel::delete(posts.filter(id.eq(post_id))).execute(connection)?;
        // let result = diesel::insert_into(posts).values(&new_post).get_result::<Post>(connection)?;
        Ok(true)
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}
