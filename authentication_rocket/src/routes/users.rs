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

use rocket_contrib::Json;
use input_types::Auth;
use controllers::users;
use super::Response;

// SIGN UP

#[post("/sign-up", format = "application/json", data = "<create_user>")]
pub fn sign_up(create_user: Json<Auth>) -> Response {
    users::sign_up(&create_user.0)
}

// LOG IN

#[post("/log-in", format = "application/json", data = "<create_user>")]
pub fn log_in(create_user: Json<Auth>) -> Response {
    users::log_in(&create_user.0)
}

// CHECK AUTHENTICATION

#[post("/is-authenticated", format = "application/json", data = "<token>")]
pub fn is_authenticated(token: Json<Auth>) -> Response {
    users::is_authenticated(&token.0)
}

// DELETE

#[post("/users/<target_user>/delete", format = "application/json", data = "<payload>")]
pub fn delete(target_user: String, payload: Json<Auth>) -> Response {
    users::delete(&target_user, &payload.0)
}

// GRANT PERMISSION

#[post("/users/<target_user>/grant/<permission>", format = "application/json", data = "<payload>")]
pub fn grant_permission(target_user: String, permission: String, payload: Json<Auth>) -> Response {
    users::grant_permission(&target_user, &permission, &payload.0)
}

// REVOKE PERMISSION

#[post("/users/<target_user>/revoke/<permission>", format = "application/json", data = "<payload>")]
pub fn revoke_permission(target_user: String, permission: String, payload: Json<Auth>) -> Response {
    users::revoke_permission(&target_user, &permission, &payload.0)
}
