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

use rocket::response::{self, Responder};
use rocket::http::Status;
use rocket::request::Request;
use rocket::Response;
use rocket_contrib::Json;
use authentication_backend::Error;
use self::error_response::ErrorResponse;

mod error_response;

#[derive(Debug)]
pub struct AuthError(Error);

impl AuthError {
    pub fn new(err: Error) -> Self {
        AuthError(err)
    }
}

impl From<Error> for AuthError {
    fn from(e: Error) -> AuthError {
        AuthError(e)
    }
}

impl ToString for AuthError {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl<'r> Responder<'r> for AuthError {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let status = match self.0 {
            Error::GetDbError => Status::InternalServerError,
            Error::NoResultError => Status::NotFound,
            Error::DieselError => Status::InternalServerError,
            Error::PasswordHashError => Status::InternalServerError,
            Error::InvalidPasswordError => Status::BadRequest,
            Error::InvalidUsernameError => Status::BadRequest,
            Error::PasswordMatchError => Status::Unauthorized,
            Error::InvalidPermissionNameError => Status::BadRequest,
            Error::PermissionError => Status::Unauthorized,
            Error::InvalidAuthError => Status::Unauthorized,
            Error::UserNotVerifiedError => Status::Unauthorized,
            Error::InvalidWebtokenError => Status::Unauthorized,
            Error::ExpiredWebtokenError => Status::Unauthorized,
            Error::ParseError => Status::InternalServerError,
            Error::IOError => Status::InternalServerError,
        };

        let json_response = Json(ErrorResponse::from_error(self.0)).respond_to(req)?;

        let response = Response::build()
            .status(status)
            .join(json_response)
            .finalize();

        Ok(response)
    }
}
