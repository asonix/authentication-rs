extern crate diesel;
use super::models::{ User, CreateUser };
use rocket_contrib::JSON;
use diesel::prelude::*;

#[post("/sign-up", format = "application/json", data = "<create_user>")]
pub fn sign_up(db: super::DB, create_user: JSON<CreateUser>) -> String {
    use schema::users;

    let insertable = match create_user.0.insertable() {
        Ok(new_user) => new_user,
        Err(m) => return json!({"message": m.to_string()}).to_string(),
    };

    let response = diesel::insert(&insertable)
        .into(users::table)
        .get_result(db.conn());


    let user: User = match response {
        Ok(user) => user,
        Err(m) => return json!({"message": m.to_string()}).to_string(),
    };

    json!({
        "message": "User Created",
        "data": {
            "id": user.id,
            "username": user.username
        }
    }).to_string()
}

#[post("/log-in")]
pub fn log_in() -> &'static str {
    "{}"
}

#[post("/is-authenticated")]
pub fn is_authenticated() -> &'static str {
    "{}"
}

#[post("/verify/<verification_token>")]
pub fn verify(verification_token: String) -> &'static str {
    "{}"
}
