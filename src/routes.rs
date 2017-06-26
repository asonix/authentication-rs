use super::models::user::{ User, CreateUser };
use rocket_contrib::JSON;

fn error_json(string: String) -> String {
    json!({
        "message": string
    }).to_string()
}

#[post("/sign-up", format = "application/json", data = "<create_user>")]
pub fn sign_up(create_user: JSON<CreateUser>) -> String {
    let new_user = match create_user.0.insertable() {
        Ok(new_user) => new_user,
        Err(m) => return error_json(m.to_string()),
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

#[post("/log-in")]
pub fn log_in() -> &'static str {
    "{}"
}

#[post("/is-authenticated")]
pub fn is_authenticated() -> &'static str {
    "{}"
}

#[get("/verify/<verification_token>")]
pub fn verify(verification_token: String) -> String {
    match User::verify_with_code(verification_token) {
        Ok(user) => {
            match user.create_webtoken() {
                Ok(token) => {
                    json!({
                        "message": "User verified",
                        "data": {
                            "token": token
                        }
                    }).to_string()
                }
                Err(_) => {
                    json!({
                        "message": "User verified",
                    }).to_string()
                }
            }
        },
        Err(m) => error_json(m),
    }
}
