use std::ops::Try;
use std::result;
use rocket::request::Request;
use rocket::response::{self, Responder};
use rocket_contrib::JSON;
use authentication_backend::models::user::User;
use error::Error;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ResponseBody {
    User { id: i32, username: String },
    Token { token: String },
    NoData,
}

impl ResponseBody {
    fn is_empty(&self) -> bool {
        match *self {
            ResponseBody::NoData => true,
            _ => false,
        }
    }
}

#[derive(Serialize)]
pub struct AuthResponse {
    message: String,
    #[serde(skip_serializing_if = "ResponseBody::is_empty")]
    data: ResponseBody,
}

impl AuthResponse {
    fn user_created(user: User) -> Self {
        AuthResponse {
            message: "User Created".to_string(),
            data: ResponseBody::User {
                id: user.id(),
                username: user.username().to_string(),
            },
        }
    }

    fn authenticated(token: Option<String>) -> Self {
        if let Some(token) = token {
            AuthResponse {
                message: "Authenticated".to_string(),
                data: ResponseBody::Token { token: token },
            }
        } else {
            AuthResponse {
                message: "Authenticated".to_string(),
                data: ResponseBody::NoData,
            }
        }
    }

    fn user_verified(token: String) -> Self {
        AuthResponse {
            message: "User Verified".to_string(),
            data: ResponseBody::Token { token: token },
        }
    }
}

pub enum AuthResult {
    Ok(JSON<AuthResponse>),
    Err(Error),
}

impl AuthResult {
    fn ok(auth_response: AuthResponse) -> AuthResult {
        AuthResult::Ok(JSON(auth_response))
    }

    pub fn user_created(user: User) -> AuthResult {
        AuthResult::ok(AuthResponse::user_created(user))
    }

    pub fn authenticated(token: Option<String>) -> AuthResult {
        AuthResult::ok(AuthResponse::authenticated(token))
    }

    pub fn user_verified(token: String) -> AuthResult {
        AuthResult::ok(AuthResponse::user_verified(token))
    }
}

impl Try for AuthResult {
    type Ok = JSON<AuthResponse>;
    type Error = Error;

    fn into_result(self) -> result::Result<Self::Ok, Self::Error> {
        match self {
            AuthResult::Ok(ok) => Ok(ok),
            AuthResult::Err(err) => Err(err),
        }
    }

    fn from_error(v: Self::Error) -> Self {
        AuthResult::Err(v)
    }

    fn from_ok(v: Self::Ok) -> Self {
        AuthResult::Ok(v)
    }
}

impl<'r> Responder<'r> for AuthResult {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        match self {
            AuthResult::Ok(json) => json.respond_to(req),
            AuthResult::Err(err) => err.respond_to(req),
        }
    }
}
