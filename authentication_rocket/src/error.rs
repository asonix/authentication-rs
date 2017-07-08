use rocket::response::{self, Responder};
use rocket::http::Status;
use rocket::request::Request;
use rocket::Response;
use rocket_contrib::JSON;
use authentication_backend::error;

pub struct Error(error::Error);

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
            error::Error::PasswordMatchError => Status::Unauthorized,
            error::Error::UserNotVerifiedError => Status::Unauthorized,
            error::Error::InvalidWebtokenError => Status::Unauthorized,
            error::Error::ExpiredWebtokenError => Status::Unauthorized,
            error::Error::ParseError => Status::InternalServerError,
            error::Error::IOError => Status::InternalServerError,
        };

        let json_response = JSON(ErrorResponse::from_error(self)).respond_to(req)?;

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
