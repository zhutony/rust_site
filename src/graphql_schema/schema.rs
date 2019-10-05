use juniper::{FieldResult, RootNode};

use std::time;

use crate::models::User;

use crate::models::{
    delete_post, get_all_posts, get_post, get_posts, get_recursive, NewPost, NewUser, Post,
};

use crate::graphql_schema::Context;

pub struct QueryRoot;

#[juniper::object(
    Context = Context,
)]

impl QueryRoot {
    fn is_logged_in(context: &Context) -> FieldResult<User> {
        let token = &context.jwt.clone();
        match token {
            Some(token) => User::from_token(token),
            None => Err("False")?,
        }
    }

    fn posts(context: &Context, parent_id: Option<i32>) -> FieldResult<Vec<Post>> {
        let connection = context.pool.get()?;
        match parent_id {
            Some(parent_id) => get_posts(&connection, parent_id),
            None => get_all_posts(&connection),
        }
    }
    fn post(context: &Context, post_id: i32) -> FieldResult<Post> {
        if post_id == 0i32 {
            let result = Post {
                id: 0i32,
                author_id: 0i32,
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
    fn login(context: &Context, username: String, password: String) -> FieldResult<String> {
        let connection = &context.pool.get()?;
        User::login(connection, Some(username), Some(password))
    }
    fn create_user(context: &Context, user: NewUser) -> FieldResult<String> {
        let connection = &context.pool.get()?;
        User::new_user(connection, user)
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}
