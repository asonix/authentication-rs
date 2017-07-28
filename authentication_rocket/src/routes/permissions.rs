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

use authentication_backend::controllers::permissions;
use rocket_contrib::Json;
use input_types::Auth;
use input_types::CreatePermission;
use super::Response;
use auth_response::AuthResponse;

#[post("/permissions", format = "application/json", data = "<new_permission>")]
pub fn create(new_permission: Json<CreatePermission>) -> Response {
    let permission = permissions::create(new_permission.0.permission(), &new_permission.0)?;

    Ok(AuthResponse::new("Permission created", permission))
}

#[post("/permissions/<permission_name>/delete", format = "application/json", data = "<payload>")]
pub fn delete(permission_name: String, payload: Json<Auth>) -> Response {
    permissions::delete(&permission_name, &payload.0)?;

    Ok(AuthResponse::empty("Permission deleted"))
}
