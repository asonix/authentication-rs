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

use super::PasswordErrorKind;
use super::UsernameErrorKind;

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
