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
use auth_result::{AuthResult, AuthResponse};
use input_types::{UserToken, UserTokenWithPassword, CreateUser, RenewalToken, GivePermission,
                  CreatePermission};
use rocket_contrib::JSON;

#[post("/sign-up", format = "application/json", data = "<create_user>")]
pub fn sign_up(create_user: JSON<CreateUser>) -> AuthResult {
    let user = User::create(&create_user.0)?;

    AuthResponse::new("User created", user).into()
}

#[post("/log-in", format = "application/json", data = "<create_user>")]
pub fn log_in(create_user: JSON<CreateUser>) -> AuthResult {
    let user = User::authenticate(&create_user.0)?;

    let token = user.create_webtoken().ok();

    AuthResponse::new("Authenticated", token).into()
}

#[post("/renew", format = "application/json", data = "<renewal_token>")]
pub fn renew(renewal_token: JSON<RenewalToken>) -> AuthResult {
    let webtoken = Webtoken::renew(&renewal_token.0.renewal_token)?;

    AuthResponse::new("Renewed", webtoken).into()
}

#[post("/is-authenticated", format = "application/json", data = "<token>")]
pub fn is_authenticated(token: JSON<UserToken>) -> AuthResult {
    User::authenticate(&token.0)?;

    AuthResponse::empty("Authenticated").into()
}

#[get("/verify/<verification_token>")]
pub fn verify(verification_token: String) -> AuthResult {
    let user = User::verify_with_code(&verification_token)?;

    let token = user.create_webtoken()?;

    AuthResponse::new("User verified", token).into()
}

#[post("/delete", format = "application/json", data = "<token_with_password>")]
pub fn delete(token_with_password: JSON<UserTokenWithPassword>) -> AuthResult {
    User::delete(&token_with_password.0)?;

    AuthResponse::empty("Deleted").into()
}

#[post("/new-permission", format = "application/json", data = "<new_permission>")]
pub fn create_permission(new_permission: JSON<CreatePermission>) -> AuthResult {
    let user = User::authenticate(&new_permission.0)?;

    if user.has_permission("admin") {
        let permission = Permission::create(new_permission.0.permission())?;

        AuthResponse::new("Permission created", permission).into()
    } else {
        Error::new(PermissionError).into()
    }
}

#[post("/give-permission", format = "application/json", data = "<give_permission>")]
pub fn give_permission(give_permission: JSON<GivePermission>) -> AuthResult {
    let authorizing_user = User::authenticate(&give_permission.0)?;

    let target_user = User::find_by_name(&give_permission.0.target_user())?;

    target_user.give_permission(
        &authorizing_user,
        &give_permission.0.permission(),
    )?;

    AuthResponse::empty("Permission granted").into()
}
