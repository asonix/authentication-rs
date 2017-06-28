use super::models::user::{ User, CreateUser };
use rocket_contrib::JSON;
use super::error::Error;

fn error_json(error: Error) -> String {
    json!({
        "message": error.to_string()
    }).to_string()
}

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

#[post("/sign-up", format = "application/json", data = "<create_user>")]
pub fn sign_up(create_user: JSON<CreateUser>) -> String {
    let new_user = match create_user.0.insertable() {
        Ok(new_user) => new_user,
        Err(m) => return error_json(m),
    };

    let user = new_user.save();

    let user: User = match user {
        Ok(user) => user,
        Err(m) => return error_json(m),
    };

    json!({
        "message": "User Created",
        "data": {
            "id": user.id(),
            "username": user.username()
        }
    }).to_string()
}

#[post("/log-in", format = "application/json", data = "<create_user>")]
pub fn log_in(create_user: JSON<CreateUser>) -> String {
    let user = match create_user.0.authenticate() {
        Ok(user) => user,
        Err(m) => return error_json(m),
    };

    let token = match user.create_webtoken() {
        Ok(token) => token,
        Err(m) => return error_json(m),
    };

    json!({
        "message": "Authenticated",
        "data": {
            "token": token
        }
    }).to_string()
}

#[post("/is-authenticated", format = "application/json", data = "<token>")]
pub fn is_authenticated(token: JSON<Token>) -> String {
    match User::from_webtoken(token.0.token) {
        Ok(_) => {
            json!({
                "message": "Authenticated"
            }).to_string()
        },
        Err(m) => error_json(m),
    }
}

#[get("/verify/<verification_token>")]
pub fn verify(verification_token: String) -> String {
    let user = match User::verify_with_code(verification_token) {
        Ok(user) => user,
        Err(m) => return error_json(m),
    };

    let token = match user.create_webtoken() {
        Ok(token) => token,
        Err(_) => json!({"message": "User verified",}).to_string(),
    };

    json!({
        "message": "User verified",
        "data": {
            "token": token
        }
    }).to_string()
}
