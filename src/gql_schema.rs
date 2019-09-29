use crate::db::MyPool;
use juniper::{FieldResult, RootNode};

use rusqlite;
use rusqlite::params;

use serde_derive::{Deserialize, Serialize};
use serde_rusqlite::*;

use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};

pub struct Context {
    pub pool: actix_web::web::Data<MyPool>,
    pub jwt: Option<String>, // pub req: actix_web::HttpRequest,
}

impl juniper::Context for Context {}

#[derive(Serialize, Deserialize, Debug, GraphQLObject, PartialEq)]
struct User {
    id: i32,
    username: String,
}

impl User {
    fn login(&self) -> FieldResult<String> {
        Ok(encode(&Header::default(), self, "secret".as_ref())?)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
            let connection = context.pool.get().unwrap();

            let result = Post {
                id: 0i32,
                content: "ROOT".to_owned(),
                parent_id: 0i32,
            };
            Ok(result)
        }
    }
    fn children(&self, context: &Context) -> FieldResult<Vec<Post>> {
        let connection = &context.pool.get().unwrap();
        let result = Post {
            id: 0i32,
            content: "ROOT".to_owned(),
            parent_id: 0i32,
        };
        Ok(vec![result])
    }
}

#[derive(GraphQLInputObject, Debug)]
struct NewPost {
    content: String,
    parent_id: i32,
}

pub struct QueryRoot;

#[juniper::object(
    Context = Context,
)]
impl QueryRoot {
    fn isLoggedIn(context: &Context) -> FieldResult<User> {
        let token = &context.jwt.clone();
        match token {
            Some(token) => {
                let user = decode::<User>(&token, "secret".as_bytes(), &Validation::default());
                Ok(user?.claims)
            }
            None => Err("False")?,
        }
    }

    fn posts(context: &Context, parent_id: Option<i32>) -> FieldResult<Vec<Post>> {
        let connection = context.pool.get().unwrap();
        match parent_id {
            Some(parent_id) => {
                let mut statement = connection
                    .prepare("SELECT * FROM posts WHERE parent_id = (?1)")
                    .unwrap();
                let mut result = from_rows::<Post>(statement.query(&[parent_id]).unwrap());
                let temp_results = result
                    .map(|a| a.expect("error getting posts"))
                    .collect::<Vec<Post>>();
                Ok(temp_results)
            }
            None => {
                let mut statement = connection.prepare("SELECT * FROM posts").unwrap();
                let mut result = from_rows::<Post>(statement.query(params![]).unwrap());
                let temp_results = result
                    .map(|a| a.expect("error getting posts"))
                    .collect::<Vec<Post>>();
                Ok(temp_results)
            }
        }

        // let result = Post {
        //     id: 0i32,
        //     content: "ROOT".to_owned(),
        //     parent_id: 0i32,
        // };
        // Ok(vec![result])
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
            let connection = context.pool.get().unwrap();
            let mut statement = connection
                .prepare("SELECT * FROM posts WHERE id = (?)")
                .unwrap();
            let mut result = from_rows::<Post>(statement.query(&[post_id]).unwrap());
            Ok(result.next().unwrap().expect("no post found"))
            // let result = Post {
            //     id: 0i32,
            //     content: "ROOT".to_owned(),
            //     parent_id: 0i32,
            // };
            // Ok(result)
        }
    }
}

pub struct MutationRoot;

#[juniper::object(
    Context = Context,
)]
impl MutationRoot {
    fn createPost(context: &Context, new_post: NewPost) -> FieldResult<bool> {
        let token = &context.jwt.clone();
        match token {
            Some(token) => {
                let token_data = decode::<User>(
                    &token,
                    "secret".as_bytes(),
                    &Validation::new(Algorithm::HS256),
                );
                let connection = &context.pool.get().unwrap();
                let mut insert_stmt = connection
                    .prepare("INSERT INTO posts (content, parent_id) VALUES (?1, ?2)")
                    .expect("failed to prepare insert post statement");
                insert_stmt
                    .execute(&[new_post.content, new_post.parent_id.to_string()])
                    .expect("error");
                Ok(true)
            }
            None => Err("Not logged in")?,
        }
    }
    fn deletePost(context: &Context, post_id: i32) -> FieldResult<bool> {
        let connection = &context.pool.get().unwrap();
        let mut insert_stmt = connection
            .prepare("DELETE FROM posts WHERE id =  (?1)")
            .expect("failed to prepare insert post statement");
        insert_stmt.execute(&[post_id.to_string()]).expect("error");
        Ok(true)
    }
    fn login(username: String, password: String) -> FieldResult<String> {
        let user = User {
            id: 0i32,
            username: username,
        };
        user.login()
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}
