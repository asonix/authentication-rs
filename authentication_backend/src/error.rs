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

use std::io;
use std::result;
use r2d2::GetTimeout;
use std::num::ParseIntError;
use bcrypt::BcryptError;
use diesel::result::Error as DbError;
use std::error::Error as StdError;
use jwt::errors::Error as JWTError;
use std::fmt;

pub type Result<T> = result::Result<T, Error>;

pub enum Error {
    BcryptError(BcryptError),
    DbError(DbError),
    InputError(InputErrorKind),
    JWTError(JWTError),
    DbTimeout,
    IOError,
    ParseError,
    PasswordMatchError,
    PermissionError,
    UserNotVerifiedError,
}

pub enum InputErrorKind {
    Password(Vec<PasswordErrorKind>),
    Username(Vec<UsernameErrorKind>),
    Authenticatable,
    PermissionName,
}

impl ToString for InputErrorKind {
    fn to_string(&self) -> String {
        match *self {
            InputErrorKind::Password(ref err_vec) => {
                let messages: Vec<String> = err_vec.iter().map(|p| p.to_string()).collect();

                messages.join(", ")
            }
            InputErrorKind::Username(ref err_vec) => {
                let messages: Vec<String> = err_vec.iter().map(|u| u.to_string()).collect();

                messages.join(", ")
            }
            InputErrorKind::Authenticatable => "Invalid authentication format".to_string(),
            InputErrorKind::PermissionName => "Invalid permission name".to_string(),
        }
    }
}

pub enum PasswordErrorKind {
    NoLowercase,
    NoNumber,
    NoSymbol,
    NoUppercase,
    TooShort,
}

impl ToString for PasswordErrorKind {
    fn to_string(&self) -> String {
        match *self {
            PasswordErrorKind::NoLowercase => "Password must contain at least one lowercase letter".to_string(),
            PasswordErrorKind::NoNumber => "Password must contain at least one number".to_string(),
            PasswordErrorKind::NoSymbol => "Password must contain at least one symbol".to_string(),
            PasswordErrorKind::NoUppercase => "Password must contain at least one uppercase letter".to_string(),
            PasswordErrorKind::TooShort => "Password must be at least 8 characters".to_string(),
        }
    }
}

pub enum UsernameErrorKind {
    Blank,
}

impl ToString for UsernameErrorKind {
    fn to_string(&self) -> String {
        match *self {
            UsernameErrorKind::Blank => "Username must not be blank".to_string(),
        }
    }
}

impl Error {
    fn input_description(input_error: &InputErrorKind) -> &str {
        match *input_error {
            InputErrorKind::Password(_) => "Invalid password",
            InputErrorKind::Username(_) => "Invalid username",
            InputErrorKind::Authenticatable => "Invalid authentication format",
            InputErrorKind::PermissionName => "Invalid permission name",
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BcryptError(ref bcrypt_error) => bcrypt_error.description(),
            Error::DbError(ref db_error) => db_error.description(),
            Error::InputError(ref input_error) => Error::input_description(input_error),
            Error::JWTError(ref jwt_error) => jwt_error.description(),
            Error::DbTimeout => "Failed to get Database",
            Error::IOError => "Timed out while waiting for database",
            Error::ParseError => "Could not parse data from string",
            Error::PasswordMatchError => "Passwords do not match",
            Error::PermissionError => "Not allowed to perform this action",
            Error::UserNotVerifiedError => "User is not verified",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::BcryptError(ref bcrypt_error) => Some(bcrypt_error),
            Error::DbError(ref db_error) => Some(db_error),
            Error::JWTError(ref jwt_error) => Some(jwt_error),
            _ => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<DbError> for Error {
    fn from(e: DbError) -> Error {
        Error::DbError(e)
    }
}

impl From<GetTimeout> for Error {
    fn from(_: GetTimeout) -> Error {
        Error::DbTimeout
    }
}

impl From<BcryptError> for Error {
    fn from(e: BcryptError) -> Error {
        Error::BcryptError(e)
    }
}

impl From<JWTError> for Error {
    fn from(e: JWTError) -> Error {
        Error::JWTError(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Error {
        Error::ParseError
    }
}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Error {
        Error::IOError
    }
}
