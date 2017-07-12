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
use error::{Result, Error};

pub fn validate_password(password: &str) -> Result<&str> {
    if password.len() < 8 {
        return Err(Error::InvalidPasswordError);
    }

    if !CONFIG.password_regex().numbers().is_match(password) {
        return Err(Error::InvalidPasswordError);
    }

    if !CONFIG.password_regex().symbols().is_match(password) {
        return Err(Error::InvalidPasswordError);
    }

    if !CONFIG.password_regex().upper().is_match(password) {
        return Err(Error::InvalidPasswordError);
    }

    if !CONFIG.password_regex().lower().is_match(password) {
        return Err(Error::InvalidPasswordError);
    }

    Ok(password)
}

pub fn validate_username(username: &str) -> Result<&str> {
    if username.len() < 1 {
        return Err(Error::InvalidUsernameError);
    }

    Ok(username)
}
