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
use std::convert::From;
use rocket::request::Request;
use rocket::response::{self, Responder};
use rocket_contrib::JSON;
use authentication_backend::models::user::User;
use authentication_backend::webtoken;
use error::Error;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ResponseBody {
    User { id: i32, username: String },
    Webtoken {
        user_token: String,
        renewal_token: String,
    },
    NoData,
}

impl From<webtoken::Webtoken> for ResponseBody {
    fn from(webtoken: webtoken::Webtoken) -> Self {
        ResponseBody::Webtoken {
            user_token: webtoken.user_token().to_owned(),
            renewal_token: webtoken.renewal_token().to_owned(),
        }
    }
}

impl From<User> for ResponseBody {
    fn from(user: User) -> Self {
        ResponseBody::User {
            id: user.id(),
            username: user.username().to_owned(),
        }
    }
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
    pub fn user_created(user: User) -> Self {
        AuthResponse {
            message: "User Created".to_owned(),
            data: user.into(),
        }
    }

    pub fn authenticated(webtoken: Option<webtoken::Webtoken>) -> Self {
        if let Some(webtoken) = webtoken {
            AuthResponse {
                message: "Authenticated".to_owned(),
                data: webtoken.into(),
            }
        } else {
            AuthResponse {
                message: "Authenticated".to_owned(),
                data: ResponseBody::NoData,
            }
        }
    }

    pub fn renewed(webtoken: webtoken::Webtoken) -> Self {
        AuthResponse {
            message: "Renewed".to_owned(),
            data: webtoken.into(),
        }
    }

    pub fn user_verified(webtoken: webtoken::Webtoken) -> Self {
        AuthResponse {
            message: "User Verified".to_owned(),
            data: webtoken.into(),
        }
    }

    pub fn deleted() -> Self {
        AuthResponse {
            message: "Deleted".to_owned(),
            data: ResponseBody::NoData,
        }
    }
}

pub enum AuthResult {
    Ok(JSON<AuthResponse>),
    Err(Error),
}

impl From<AuthResponse> for AuthResult {
    fn from(auth_response: AuthResponse) -> Self {
        AuthResult::Ok(JSON(auth_response))
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
