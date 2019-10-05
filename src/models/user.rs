use crate::db::MyPooledConnection;

use juniper::FieldResult;

use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};

use rusqlite::{self, params};

use serde_rusqlite::*;

use serde_derive::{Deserialize, Serialize};

use std::env;

use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Serialize, Deserialize, Clone, Debug, GraphQLObject, PartialEq)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub hash: String,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub exp: i32,
}

#[derive(GraphQLInputObject, Debug)]
pub struct NewUser {
    pub email: String,
    pub hash: String,
    pub username: String,
    pub password: String,
    pub password_confirm: String,
    pub firstname: String,
    pub lastname: String,
}

impl User {
    pub fn login(
        connection: &MyPooledConnection,
        username: Option<String>,
        password: Option<String>,
    ) -> FieldResult<String> {
        let token = match username {
            Some(username) => {
                let mut statement =
                    connection.prepare("SELECT * FROM users WHERE id = (?) LIMIT 1")?;
                let mut result = from_rows::<User>(statement.query(&[username])?);
                let user = result.next();
                match user {
                    Some(user) => {
                        match password {
                            Some(password) => {
                                let user = &user?;
                                let valid = verify(&password, &user.hash)?;
                                if valid == true {
                                    let mut header = Header::default();
                                    // header.kid = Some("signing_key".to_owned());
                                    header.alg = Algorithm::HS256;
                                    let key = match env::var("JWT_SECRET") {
                                        Ok(env) => env,
                                        Err(_err) => "secret".to_owned(), // really really dumb should swap to a random key on startup atleast
                                    };
                                    encode::<User>(&header, &user, key.as_ref())?
                                } else {
                                    Err("password invalid")?
                                }
                            }
                            None => Err("no password found found")?,
                        }
                    }
                    None => Err("no user found")?,
                }
            }
            None => Err("no username supplied")?,
        };
        Ok(token)
    }
    pub fn from_token(token: &String) -> FieldResult<User> {
        let mut validation = Validation::default();
        validation.algorithms = vec![Algorithm::HS256];
        let key = match env::var("JWT_SECRET") {
            Ok(env) => env,
            Err(_err) => "secret".to_owned(), // really really dumb should swap to a random key on startup atleast
        };
        let user = decode::<User>(&token, key.as_bytes(), &validation);
        Ok(user?.claims)
    }
    pub fn new_user(connection: &MyPooledConnection, user: NewUser) -> FieldResult<String> {
        let mut insert_stmt = connection.prepare(
            "INSERT INTO users ( username,
                    email,
                    hash, 
                    firstname, 
                    lastname) VALUES(?1, ?2, ?3, ?4, ?5)",
        )?;
        if user.password != user.password_confirm {
            Err("passwords are not the same")?
        }
        let hash = hash(user.password, DEFAULT_COST)?;
        insert_stmt.execute(&[
            user.email.to_owned(),
            hash.to_owned(),
            user.firstname.to_owned(),
            user.lastname.to_owned(),
        ])?;
        let temp_user = User {
            id: 1,
            hash: hash.to_owned(),
            email: user.email.to_owned(),
            username: user.username.to_owned(),
            firstname: user.firstname.to_owned(),
            lastname: user.lastname.to_owned(),
            exp: 100000,
        };

        let mut header = Header::default();
        // header.kid = Some("signing_key".to_owned());
        header.alg = Algorithm::HS256;
        let key = match env::var("JWT_SECRET") {
            Ok(env) => env,
            Err(_err) => "secret".to_owned(), // really really dumb should swap to a random key on startup atleast
        };

        let token = encode::<User>(&header, &temp_user, key.as_ref())?;
        Ok(token)
    }
}
