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

use CONFIG;
use error::Result;
use error::Error::InputError;
use error::InputErrorKind::{Username, Password};
use error::UsernameErrorKind;
use error::UsernameErrorKind::Blank;
use error::PasswordErrorKind;
use error::PasswordErrorKind::{TooShort, NoNumber, NoSymbol, NoUppercase, NoLowercase};

pub fn validate_password(password: &str) -> Result<&str> {
    let mut error_vec: Vec<PasswordErrorKind> = Vec::new();

    if password.len() < 8 {
        error_vec.push(TooShort);
    }

    if !CONFIG.password_regex().numbers().is_match(password) {
        error_vec.push(NoNumber);
    }

    if !CONFIG.password_regex().symbols().is_match(password) {
        error_vec.push(NoSymbol);
    }

    if !CONFIG.password_regex().upper().is_match(password) {
        error_vec.push(NoUppercase);
    }

    if !CONFIG.password_regex().lower().is_match(password) {
        error_vec.push(NoLowercase);
    }

    if error_vec.len() == 0 {
        Ok(password)
    } else {
        Err(InputError(Password(error_vec)))
    }
}

pub fn validate_username(username: &str) -> Result<&str> {
    let mut error_vec: Vec<UsernameErrorKind> = Vec::new();

    if username.len() == 0 {
        error_vec.push(Blank)
    }

    if error_vec.len() == 0 {
        Ok(username)
    } else {
        Err(InputError(Username(error_vec)))
    }
}
