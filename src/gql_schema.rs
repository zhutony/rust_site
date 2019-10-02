use crate::db::{MyPool, MyPooledConnection};
use juniper::{FieldResult, RootNode};

use rusqlite;
use rusqlite::params;

use serde_derive::{Deserialize, Serialize};
use serde_rusqlite::*;

use std::env;
use std::time;

use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};

pub struct Context {
    pub pool: actix_web::web::Data<MyPool>,
    pub jwt: Option<String>,
}

impl juniper::Context for Context {}

#[derive(Serialize, Deserialize, Debug, GraphQLObject, PartialEq)]
struct User {
    id: i32,
    username: String,
    exp: i32,
}

impl User {
    fn login(&self) -> FieldResult<String> {
        let mut header = Header::default();
        // header.kid = Some("signing_key".to_owned());
        header.alg = Algorithm::HS256;
        let key = match env::var("JWT_SECRET") {
            Ok(env) => env,
            Err(_err) => "secret".to_owned(), // really really dumb should swap to a random key on startup atleast
        };
        Ok(encode(&header, self, key.as_ref())?)
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
            get_post(&context.pool.get()?, self.parent_id)
        }
    }
    fn children(&self, context: &Context) -> FieldResult<Vec<Post>> {
        get_posts(&context.pool.get()?, self.id)
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
    fn is_logged_in(context: &Context) -> FieldResult<User> {
        let token = &context.jwt.clone();
        match token {
            Some(token) => {
                let mut validation = Validation::default();
                validation.algorithms = vec![Algorithm::HS256];
                let key = match env::var("JWT_SECRET") {
                    Ok(env) => env,
                    Err(err) => "secret".to_owned(), // really really dumb should swap to a random key on startup atleast
                };
                let user = decode::<User>(&token, key.as_bytes(), &validation);
                Ok(user?.claims)
            }
            None => Err("False")?,
        }
    }

    fn posts(context: &Context, parent_id: Option<i32>) -> FieldResult<Vec<Post>> {
        let connection = context.pool.get()?;
        match parent_id {
            Some(parent_id) => get_posts(&connection, parent_id),
            None => {
                let now = time::Instant::now();
                let mut statement = connection.prepare("SELECT * FROM posts")?;
                let mut result = from_rows::<Post>(statement.query(params![])?);
                let temp_results = result.collect::<Result<Vec<_>>>();
                println!("time taken {:?}", now.elapsed());
                Ok(temp_results?)
            }
        }
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
            get_post(&context.pool.get()?, post_id)
        }
    }
}

pub struct MutationRoot;

#[juniper::object(
    Context = Context,
)]
impl MutationRoot {
    fn create_post(context: &Context, new_post: NewPost) -> FieldResult<bool> {
        let token = &context.jwt.clone();
        match token {
            Some(token) => {
                let key = match env::var("JWT_SECRET") {
                    Ok(env) => env,
                    Err(err) => "secret".to_owned(), // really really dumb should swap to a random key on startup atleast
                };
                let token_data =
                    decode::<User>(&token, key.as_bytes(), &Validation::new(Algorithm::HS256));
                let connection = &context.pool.get()?;
                let mut insert_stmt =
                    connection.prepare("INSERT INTO posts (content, parent_id) VALUES (?1, ?2)")?;
                let mut statements = "".to_owned();
                let now = time::Instant::now();

                for x in 0..100000 {
                    statements = statements
                        + "INSERT INTO posts (content, parent_id) VALUES (\"speed\", 1);";
                }
                connection.execute_batch(&format!(
                    "
                        BEGIN TRANSACTION;
                        {}
                        COMMIT;
                    ",
                    statements
                ))?;
                println!("time taken {:?}", now.elapsed());

                Ok(true)
            }
            None => Err("Not logged in")?,
        }
    }
    pub fn delete_post(context: &Context, post_id: i32) -> FieldResult<bool> {
        delete_post(&context.pool.get()?, post_id)
    }
    fn delete_posts_recursive(&self, context: &Context, post_id: i32) -> FieldResult<bool> {
        let connection = &context.pool.get()?;
        let mut posts = get_recursive(connection, post_id, 3)?;
        posts.push(get_post(connection, post_id)?);
        for post in posts {
            delete_post(connection, post.id)?;
        }
        Ok(true)
    }
    fn login(username: String, password: String) -> FieldResult<String> {
        let user = User {
            id: 0i32,
            username: username,
            exp: std::i32::MAX,
        };
        user.login()
    }
}

fn delete_post(connection: &MyPooledConnection, post_id: i32) -> FieldResult<bool> {
    let mut insert_stmt = connection.prepare("DELETE FROM posts WHERE id =  (?1)")?;
    insert_stmt.execute(&[post_id.to_string()])?;
    Ok(true)
}

fn get_post(connection: &MyPooledConnection, post_id: i32) -> FieldResult<Post> {
    let mut statement = connection.prepare("SELECT * FROM posts WHERE id = (?) LIMIT 1")?;
    let mut result = from_rows::<Post>(statement.query(&[post_id.to_string()])?);
    let post = result.next();
    match post {
        Some(post) => Ok(post?),
        None => Err(format!("No posts found with id {}", post_id))?,
    }
}
fn get_posts(connection: &MyPooledConnection, parent_id: i32) -> FieldResult<Vec<Post>> {
    let mut statement = connection.prepare("SELECT * FROM posts WHERE parent_id = (?1)")?;
    let result = from_rows::<Post>(statement.query(&[parent_id.to_string()])?);
    let temp_results = result.collect::<Result<Vec<_>>>();
    Ok(temp_results?)
}
fn get_recursive(
    connection: &MyPooledConnection,
    post_id: i32,
    depth: i32,
) -> FieldResult<Vec<Post>> {
    let mut posts = get_posts(connection, post_id)?;
    let mut temp_results = vec![];
    temp_results.append(&mut posts);
    if depth != 0 {
        for post in posts {
            temp_results.append(&mut get_recursive(connection, post.id, depth - 1)?);
        }
    }
    Ok(temp_results)
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}
