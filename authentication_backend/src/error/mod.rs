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

mod result;
mod error;
mod input_error_kind;
mod password_error_kind;
mod username_error_kind;

pub use bcrypt::BcryptError;
pub use diesel::result::Error as DbError;
pub use diesel::result::DatabaseErrorKind as DbErrorKind;
pub use jwt::errors::Error as JWTError;
pub use jwt::errors::ErrorKind as JWTErrorKind;

pub use self::result::Result;
pub use self::error::Error;
pub use self::input_error_kind::InputErrorKind;
pub use self::password_error_kind::PasswordErrorKind;
pub use self::username_error_kind::UsernameErrorKind;
