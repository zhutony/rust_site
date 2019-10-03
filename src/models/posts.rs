use crate::db::MyPooledConnection;
use crate::graphql_schema::Post;

use rusqlite::{self, params};
use serde_rusqlite::*;

use juniper::FieldResult;

use std::time;

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
    let mut result = from_rows::<Post>(statement.query(params![])?);
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
