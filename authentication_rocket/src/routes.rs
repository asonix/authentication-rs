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

use authentication_backend::models::user::User;
use auth_result::AuthResult;
use input_types::{ToAuth, Token, TokenWithPassword, CreateUser};
use rocket_contrib::JSON;

#[post("/sign-up", format = "application/json", data = "<create_user>")]
pub fn sign_up(create_user: JSON<CreateUser>) -> AuthResult {
    let user = User::create(&create_user.0.to_auth())?;

    AuthResult::user_created(user)
}

#[post("/log-in", format = "application/json", data = "<create_user>")]
pub fn log_in(create_user: JSON<CreateUser>) -> AuthResult {
    let user = User::authenticate(&create_user.0.to_auth())?;

    let token = user.create_webtoken().ok();

    AuthResult::authenticated(token)
}

#[post("/is-authenticated", format = "application/json", data = "<token>")]
pub fn is_authenticated(token: JSON<Token>) -> AuthResult {
    User::authenticate(&token.0.to_auth())?;

    AuthResult::authenticated(None)
}

#[get("/verify/<verification_token>")]
pub fn verify(verification_token: String) -> AuthResult {
    let user = User::verify_with_code(&verification_token)?;

    let token = user.create_webtoken()?;

    AuthResult::user_verified(token)
}

#[post("/delete", format = "application/json", data = "<token_with_password>")]
pub fn delete(token_with_password: JSON<TokenWithPassword>) -> AuthResult {
    User::delete(&token_with_password.0.to_auth())?;

    AuthResult::deleted()
}
