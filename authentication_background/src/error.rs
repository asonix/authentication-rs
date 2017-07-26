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

use std::error::Error as StdError;
use std::any::Any;
use std::fmt;
use std::sync::mpsc;
use super::Message;

#[derive(Debug)]
pub enum Error {
    ProcessingError(String),
    DuplicateHandler(String),
    ExitHandler,
    SendError,
    JoinError,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ProcessingError(_) => "Error processing job",
            Error::DuplicateHandler(_) => "Handler with that name already exists",
            Error::ExitHandler => "Cannot register handler with reserved anme 'exit'",
            Error::SendError => "Could not send data",
            Error::JoinError => "Could not join thread",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ProcessingError(ref s) => write!(f, "Error processing data: '{}'", s),
            Error::DuplicateHandler(ref s) => write!(f, "Handler already exists for '{}'", s),
            Error::ExitHandler => write!(f, "Cannot register handler with reserved name 'exit'"),
            Error::SendError => write!(f, "Could not send data to thread"),
            Error::JoinError => write!(f, "Could not join thread"),
        }
    }
}

impl<T> From<mpsc::SendError<Message<T>>> for Error {
    fn from(_err: mpsc::SendError<Message<T>>) -> Error {
        Error::SendError
    }
}

impl From<Box<Any + Send>> for Error {
    fn from(_err: Box<Any + Send>) -> Error {
        Error::JoinError
    }
}
