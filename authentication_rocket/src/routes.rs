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

use authentication_backend::models::permission::Permission;
use authentication_backend::models::user::User;
use authentication_backend::webtoken::Webtoken;
use authentication_backend::error::Error::PermissionError;
use error::Error;
use auth_result::AuthResponse;
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
    let user = User::authenticate(&create_user.0)?;

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
    let user = User::verify_with_code(&verification_token)?;

    let token = user.create_webtoken()?;

    Ok(AuthResponse::new("User verified", token))
}

#[post("/delete", format = "application/json", data = "<token_with_password>")]
pub fn delete(token_with_password: Json<UserTokenWithPassword>) -> Response {
    User::delete(&token_with_password.0)?;

    Ok(AuthResponse::empty("Deleted"))
}

#[post("/new-permission", format = "application/json", data = "<new_permission>")]
pub fn create_permission(new_permission: Json<CreatePermission>) -> Response {
    let user = User::authenticate(&new_permission.0)?;

    if user.has_permission("admin") {
        let permission = Permission::create(new_permission.0.permission())?;

        Ok(AuthResponse::new("Permission created", permission))
    } else {
        Err(Error::new(PermissionError))
    }
}

#[post("/give-permission", format = "application/json", data = "<give_permission>")]
pub fn give_permission(give_permission: Json<GivePermission>) -> Response {
    let authorizing_user = User::authenticate(&give_permission.0)?;

    let target_user = User::find_by_name(&give_permission.0.target_user())?;

    target_user.give_permission(
        &authorizing_user,
        &give_permission.0.permission(),
    )?;

    Ok(AuthResponse::empty("Permission granted"))
}
