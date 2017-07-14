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
use authentication_backend::error;

#[derive(Debug)]
pub struct Error(error::Error);

impl Error {
    pub fn new(err: error::Error) -> Self {
        Error(err)
    }
}

impl From<error::Error> for Error {
    fn from(e: error::Error) -> Error {
        Error(e)
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let status = match self.0 {
            error::Error::GetDbError => Status::InternalServerError,
            error::Error::NoResultError => Status::NotFound,
            error::Error::DieselError => Status::InternalServerError,
            error::Error::PasswordHashError => Status::InternalServerError,
            error::Error::InvalidPasswordError => Status::BadRequest,
            error::Error::InvalidUsernameError => Status::BadRequest,
            error::Error::PasswordMatchError => Status::Unauthorized,
            error::Error::InvalidPermissionNameError => Status::BadRequest,
            error::Error::PermissionError => Status::Unauthorized,
            error::Error::InvalidAuthError => Status::Unauthorized,
            error::Error::UserNotVerifiedError => Status::Unauthorized,
            error::Error::InvalidWebtokenError => Status::Unauthorized,
            error::Error::ExpiredWebtokenError => Status::Unauthorized,
            error::Error::ParseError => Status::InternalServerError,
            error::Error::IOError => Status::InternalServerError,
        };

        let json_response = Json(ErrorResponse::from_error(self)).respond_to(req)?;

        let response = Response::build()
            .status(status)
            .join(json_response)
            .finalize();

        Ok(response)
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    message: String,
}

impl ErrorResponse {
    fn from_error(error: Error) -> Self {
        ErrorResponse { message: error.to_string() }
    }
}
