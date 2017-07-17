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

use authentication_backend::{ToAuth, Admin, User};
use routes::Response;
use auth_response::AuthResponse;

pub fn create<T>(permission: &str, auth: &T) -> Response
where
    T: ToAuth,
{
    let user = User::authenticate(auth)?;
    let admin = Admin::from_authenticated(user)?;

    let permission = admin.create_permission(permission)?;

    Ok(AuthResponse::new("Permission created", permission))
}

pub fn delete<T>(permission: &str, auth: &T) -> Response
where
    T: ToAuth,
{
    let user = User::authenticate(auth)?;
    let admin = Admin::from_authenticated(user)?;

    admin.delete_permission(permission)?;

    Ok(AuthResponse::empty("Permission deleted"))
}
