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

use authentication_backend::controllers::users;
use authentication_backend::Error as BackendError;
use authentication_background::MsgSender;
use rocket_contrib::Json;
use rocket::State;
use std::sync::Mutex;
use input_types::Auth;
use super::Response;
use auth_response::AuthResponse;

// SIGN UP

#[post("/sign-up", format = "application/json", data = "<create_user>")]
pub fn sign_up(create_user: Json<Auth>, sender: State<Mutex<MsgSender<i32>>>) -> Response {
    let sender = match sender.lock() {
        Ok(sender) => sender.clone(),
        Err(_) => return Err(BackendError::IOError.into()),
    };

    let user = users::sign_up(&create_user.0, &sender)?;

    Ok(AuthResponse::new("User created", Some(user)))
}

// LOG IN

#[post("/log-in", format = "application/json", data = "<create_user>")]
pub fn log_in(create_user: Json<Auth>) -> Response {
    let token = users::log_in(&create_user.0)?;

    Ok(AuthResponse::new("Authenticated", token))
}

// CHECK AUTHENTICATION

#[post("/is-authenticated", format = "application/json", data = "<token>")]
pub fn is_authenticated(token: Json<Auth>) -> Response {
    users::is_authenticated(&token.0)?;

    Ok(AuthResponse::empty("Authenticated"))
}

// DELETE

#[post("/users/<target_user>/delete", format = "application/json", data = "<payload>")]
pub fn delete(target_user: String, payload: Json<Auth>) -> Response {
    users::delete(&target_user, &payload.0)?;

    Ok(AuthResponse::empty("Deleted"))
}

// GRANT PERMISSION

#[post("/users/<target_user>/grant/<permission>", format = "application/json", data = "<payload>")]
pub fn grant_permission(target_user: String, permission: String, payload: Json<Auth>) -> Response {
    users::grant_permission(&target_user, &permission, &payload.0)?;

    Ok(AuthResponse::empty("Permission granted"))
}

// REVOKE PERMISSION

#[post("/users/<target_user>/revoke/<permission>", format = "application/json", data = "<payload>")]
pub fn revoke_permission(target_user: String, permission: String, payload: Json<Auth>) -> Response {
    users::revoke_permission(&target_user, &permission, &payload.0)?;

    Ok(AuthResponse::empty("Permission revoked"))
}
