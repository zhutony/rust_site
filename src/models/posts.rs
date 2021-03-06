use crate::db::MyPooledConnection;
use crate::graphql_schema::Context;

use serde_derive::{Deserialize, Serialize};

use rusqlite::{self, params};
use serde_rusqlite::*;

use juniper::FieldResult;

use std::time;

use crate::models::User;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Post {
    pub id: i32,
    pub content: String,
    pub author_id: i32,
    pub parent_id: i32,
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
    fn author_id(&self) -> i32 {
        self.author_id
    }
    fn author(&self, context: &Context) -> FieldResult<User> {
        let temp_author_id = self.author_id.clone();
        if temp_author_id == 0i32 {
            let result = User {
                id: 0i32,
                email: "admin@domain.com".to_owned(),
                firstname: "admin".to_owned(),
                lastname: "admin".to_owned(),
                username: "admin".to_owned(),
                hash: "nohash".to_owned(),
            };
            Ok(result)
        } else {
            User::get_user(&context.pool.get()?, None, None, Some(self.author_id))
        }
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
                author_id: 0i32,
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
pub struct NewPost {
    content: String,
    author_id: i32,
    parent_id: i32,
}

pub fn delete_post(connection: &MyPooledConnection, post_id: i32) -> FieldResult<bool> {
    let mut insert_stmt = connection.prepare("DELETE FROM posts WHERE id =  (?1)")?;
    insert_stmt.execute(&[post_id.to_string()])?;
    Ok(true)
}

pub fn get_post(connection: &MyPooledConnection, post_id: i32) -> FieldResult<Post> {
    let mut statement = connection.prepare("SELECT * FROM posts WHERE id = (?) LIMIT 1")?;
    let mut result = from_rows::<Post>(statement.query(&[post_id.to_string()])?);
    let post = result.next();
    match post {
        Some(post) => Ok(post?),
        None => Err(format!("No posts found with id {}", post_id))?,
    }
}
pub fn get_posts(connection: &MyPooledConnection, parent_id: i32) -> FieldResult<Vec<Post>> {
    let mut statement = connection.prepare("SELECT * FROM posts WHERE parent_id = (?1)")?;
    let result = from_rows::<Post>(statement.query(&[parent_id.to_string()])?);
    let temp_results = result.collect::<Result<Vec<_>>>();
    Ok(temp_results?)
}

pub fn get_all_posts(connection: &MyPooledConnection) -> FieldResult<Vec<Post>> {
    let now = time::Instant::now();
    let mut statement = connection.prepare("SELECT * FROM posts")?;
    let result = from_rows::<Post>(statement.query(params![])?);
    let temp_results = result.collect::<Result<Vec<_>>>();
    println!("time taken {:?}", now.elapsed());
    Ok(temp_results?)
}

pub fn get_recursive(
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
