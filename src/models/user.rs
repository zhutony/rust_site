use juniper::FieldResult;

use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};

use serde_derive::{Deserialize, Serialize};

use std::env;

#[derive(Serialize, Deserialize, Debug, GraphQLObject, PartialEq)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub exp: i32,
}

impl User {
    pub fn login(&self) -> FieldResult<String> {
        let mut header = Header::default();
        // header.kid = Some("signing_key".to_owned());
        header.alg = Algorithm::HS256;
        let key = match env::var("JWT_SECRET") {
            Ok(env) => env,
            Err(_err) => "secret".to_owned(), // really really dumb should swap to a random key on startup atleast
        };
        Ok(encode(&header, self, key.as_ref())?)
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
}
