extern crate diesel;
extern crate r2d2;
extern crate bcrypt;
extern crate frank_jwt;

use std::io;
use std::result;
use std::num;
use rocket::response::{self, Responder};
use rocket::http::Status;

pub enum Error {
    GetDbError,
    NoResultError,
    DieselError,
    PasswordHashError,
    InvalidPasswordError,
    PasswordMatchError,
    UserNotVerifiedError,
    InvalidWebtokenError,
    ExpiredWebtokenError,
    ParseError,
    IOError,
}

pub type Result<T> = result::Result<T, Error>;

impl<'r> Responder<'r> for Error {
    fn respond(self) -> response::Result<'r> {
        let status = match self {
            Error::GetDbError => Status::InternalServerError,
            Error::NoResultError => Status::NotFound,
            Error::DieselError => Status::InternalServerError,
            Error::PasswordHashError => Status::InternalServerError,
            Error::InvalidPasswordError => Status::BadRequest,
            Error::PasswordMatchError => Status::Unauthorized,
            Error::UserNotVerifiedError => Status::Unauthorized,
            Error::InvalidWebtokenError => Status::Unauthorized,
            Error::ExpiredWebtokenError => Status::Unauthorized,
            Error::ParseError => Status::InternalServerError,
            Error::IOError => Status::InternalServerError,
        };

        Err(status)
    }
}


impl ToString for Error {
    fn to_string(&self) -> String {
        match *self {
            Error::GetDbError => "Timed out while waiting for database".to_string(),
            Error::NoResultError => "Could not find requested resource".to_string(),
            Error::DieselError => "Invalid database interaction".to_string(),
            Error::PasswordHashError => "Could not hash password".to_string(),
            Error::InvalidPasswordError => "Password did not meet requirements".to_string(),
            Error::PasswordMatchError => "Passwords do not match".to_string(),
            Error::UserNotVerifiedError => "User is not verified".to_string(),
            Error::InvalidWebtokenError => "Webtoken is invalid".to_string(),
            Error::ExpiredWebtokenError => "Webtoken has expired".to_string(),
            Error::ParseError => "Could not parse data from string".to_string(),
            Error::IOError => "Something went wrong".to_string(),
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Error {
        match e {
            diesel::result::Error::NotFound => Error::NoResultError,
            _ => Error::DieselError,
        }
    }
}

impl From<r2d2::GetTimeout> for Error {
    fn from(_: r2d2::GetTimeout) -> Error {
        Error::GetDbError
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(_: bcrypt::BcryptError) -> Error {
        Error::PasswordHashError
    }
}

impl From<frank_jwt::Error> for Error {
    fn from(e: frank_jwt::Error) -> Error {
        match e {
            frank_jwt::Error::SignatureExpired => Error::ExpiredWebtokenError,
            _ => Error::InvalidWebtokenError,
        }
    }
}

impl From<num::ParseIntError> for Error {
    fn from(_: num::ParseIntError) -> Error {
        Error::ParseError
    }
}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Error {
        Error::IOError
    }
}
