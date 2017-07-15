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
use authentication_backend::Error as BackendError;
use authentication_backend::{BcryptError, DbError, DbErrorKind, JWTError, JWTErrorKind};
use self::error_response::ErrorResponse;

mod error_response;

#[derive(Debug)]
pub struct Error(BackendError);

impl Error {
    fn bcrypt_status(_err: &BcryptError) -> Status {
        Status::InternalServerError
    }

    fn db_kind_status(err: &DbErrorKind) -> Status {
        match *err {
            DbErrorKind::UniqueViolation => Status::BadRequest,
            DbErrorKind::ForeignKeyViolation => Status::BadRequest,
            DbErrorKind::UnableToSendCommand => Status::InternalServerError,
            _ => Status::InternalServerError,
        }
    }

    fn db_status(err: &DbError) -> Status {
        match *err {
            DbError::InvalidCString(_) => Status::InternalServerError,
            DbError::DatabaseError(ref err, _) => Error::db_kind_status(err),
            DbError::NotFound => Status::BadRequest,
            DbError::QueryBuilderError(_) => Status::InternalServerError,
            DbError::DeserializationError(_) => Status::InternalServerError,
            DbError::SerializationError(_) => Status::InternalServerError,
            _ => Status::InternalServerError,
        }
    }

    fn jwt_status(err: &JWTError) -> Status {
        match *err.kind() {
            JWTErrorKind::InvalidToken => Status::BadRequest,
            JWTErrorKind::InvalidSignature => Status::BadRequest,
            JWTErrorKind::InvalidKey => Status::InternalServerError,
            JWTErrorKind::ExpiredSignature => Status::Unauthorized,
            JWTErrorKind::InvalidIssuer => Status::Unauthorized,
            JWTErrorKind::InvalidAudience => Status::Unauthorized,
            JWTErrorKind::InvalidSubject => Status::Unauthorized,
            JWTErrorKind::InvalidIssuedAt => Status::Unauthorized,
            JWTErrorKind::ImmatureSignature => Status::Unauthorized,
            JWTErrorKind::InvalidAlgorithm => Status::Unauthorized,
            _ => Status::InternalServerError,
        }
    }
}

impl From<BackendError> for Error {
    fn from(e: BackendError) -> Error {
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
            BackendError::BcryptError(ref err) => Error::bcrypt_status(err),
            BackendError::DbError(ref err) => Error::db_status(err),
            BackendError::InputError(_) => Status::BadRequest,
            BackendError::JWTError(ref err) => Error::jwt_status(err),
            BackendError::DbTimeout => Status::InternalServerError,
            BackendError::IOError => Status::InternalServerError,
            BackendError::ParseError => Status::InternalServerError,
            BackendError::PasswordMatchError => Status::Unauthorized,
            BackendError::PermissionError => Status::Unauthorized,
            BackendError::UserNotVerifiedError => Status::Unauthorized,
        };

        let json_response = Json(ErrorResponse::from_error(self.0)).respond_to(req)?;

        let response = Response::build()
            .status(status)
            .join(json_response)
            .finalize();

        Ok(response)
    }
}
