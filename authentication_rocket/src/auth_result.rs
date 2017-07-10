/*
 * This file is part of Authentication.
 *
 * Copyright © 2017 Riley Trautman
 *
 * Authentication is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Authentication is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Authentication.  If not, see <http://www.gnu.org/licenses/>.
 */

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

    fn deleted() -> Self {
        AuthResponse {
            message: "Deleted".to_string(),
            data: ResponseBody::NoData,
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

    pub fn deleted() -> AuthResult {
        AuthResult::ok(AuthResponse::deleted())
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
