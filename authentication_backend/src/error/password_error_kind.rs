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
            PasswordErrorKind::NoLowercase => {
                "Password must contain at least one lowercase letter".to_string()
            }
            PasswordErrorKind::NoNumber => "Password must contain at least one number".to_string(),
            PasswordErrorKind::NoSymbol => "Password must contain at least one symbol".to_string(),
            PasswordErrorKind::NoUppercase => {
                "Password must contain at least one uppercase letter".to_string()
            }
            PasswordErrorKind::TooShort => "Password must be at least 8 characters".to_string(),
        }
    }
}
