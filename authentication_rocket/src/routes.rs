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

use authentication_backend::{Admin, User, Webtoken};
use error::Error;
use auth_response::AuthResponse;
use input_types::{UserToken, UserTokenWithPassword, CreateUser, RenewalToken, GivePermission,
                  CreatePermission};
use rocket_contrib::Json;

type Response = Result<AuthResponse, Error>;

#[post("/sign-up", format = "application/json", data = "<create_user>")]
pub fn sign_up(create_user: Json<CreateUser>) -> Response {
    let user = User::create(&create_user.0)?;

    Ok(AuthResponse::new("User created", user))
}

#[post("/log-in", format = "application/json", data = "<create_user>")]
pub fn log_in(create_user: Json<CreateUser>) -> Response {
    let user = User::authenticate_session(&create_user.0)?;

    let token = user.create_webtoken().ok();

    Ok(AuthResponse::new("Authenticated", token))
}

#[post("/renew", format = "application/json", data = "<renewal_token>")]
pub fn renew(renewal_token: Json<RenewalToken>) -> Response {
    let webtoken = Webtoken::renew(&renewal_token.0.renewal_token)?;

    Ok(AuthResponse::new("Renewed", webtoken))
}

#[post("/is-authenticated", format = "application/json", data = "<token>")]
pub fn is_authenticated(token: Json<UserToken>) -> Response {
    User::authenticate(&token.0)?;

    Ok(AuthResponse::empty("Authenticated"))
}

#[get("/verify/<verification_token>")]
pub fn verify(verification_token: String) -> Response {
    User::verify_with_code(&verification_token)?;

    Ok(AuthResponse::empty("User verified"))
}

#[post("/delete", format = "application/json", data = "<token_with_password>")]
pub fn delete(token_with_password: Json<UserTokenWithPassword>) -> Response {
    let user = User::authenticate_session(&token_with_password.0)?;

    user.delete()?;

    Ok(AuthResponse::empty("Deleted"))
}

#[post("/new-permission", format = "application/json", data = "<new_permission>")]
pub fn create_permission(new_permission: Json<CreatePermission>) -> Response {
    let user = User::authenticate(&new_permission.0)?;
    let admin = Admin::from_authenticated(user)?;

    let permission = admin.create_permission(new_permission.0.permission())?;

    Ok(AuthResponse::new("Permission created", permission))
}

#[post("/give-permission/<target_user>", format = "application/json", data = "<payload>")]
pub fn give_permission(target_user: String, payload: Json<GivePermission>) -> Response {
    let user = User::authenticate(&payload.0)?;
    let admin = Admin::from_authenticated(user)?;

    let target_user = User::find_by_name(&target_user)?;

    admin.give_permission(&target_user, &payload.0.permission())?;

    Ok(AuthResponse::empty("Permission granted"))
}

#[post("/revoke-permission/<target_user>", format = "application/json", data = "<payload>")]
pub fn revoke_permission(target_user: String, payload: Json<GivePermission>) -> Response {
    let user = User::authenticate(&payload.0)?;
    let admin = Admin::from_authenticated(user)?;

    let target_user = User::find_by_name(&target_user)?;

    admin.revoke_permission(
        &target_user,
        &payload.0.permission(),
    )?;

    Ok(AuthResponse::empty("Permission revoked"))
}
