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
use rocket_contrib::Json;
use authentication_backend::models::user::User;
use authentication_backend::models::permission::Permission;
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
    Permission { id: i32, name: String },
    NoData,
}

impl From<Permission> for ResponseBody {
    fn from(permission: Permission) -> Self {
        ResponseBody::Permission {
            id: permission.id(),
            name: permission.name().to_owned(),
        }
    }
}

impl<T: Into<ResponseBody>> From<Option<T>> for ResponseBody {
    fn from(option: Option<T>) -> Self {
        if let Some(value) = option {
            value.into()
        } else {
            ResponseBody::NoData
        }
    }
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
    pub fn new<T>(message: &str, data: T) -> Self
    where
        T: Into<ResponseBody>,
    {
        AuthResponse {
            message: message.to_owned(),
            data: data.into(),
        }
    }

    pub fn empty(message: &str) -> Self {
        AuthResponse {
            message: message.to_owned(),
            data: ResponseBody::NoData,
        }
    }
}

pub enum AuthResult {
    Ok(Json<AuthResponse>),
    Err(Error),
}

impl From<AuthResponse> for AuthResult {
    fn from(auth_response: AuthResponse) -> Self {
        AuthResult::Ok(Json(auth_response))
    }
}

impl From<Error> for AuthResult {
    fn from(err: Error) -> Self {
        AuthResult::Err(err)
    }
}

impl Try for AuthResult {
    type Ok = Json<AuthResponse>;
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
