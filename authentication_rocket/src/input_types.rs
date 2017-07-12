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

use authentication_backend::authenticatable::{Authenticatable, ToAuth};

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

impl ToAuth for CreateUser {
    fn to_auth(&self) -> Authenticatable {
        Authenticatable::UserAndPass {
            username: &self.username,
            password: &self.password,
        }
    }
}

#[derive(Deserialize)]
pub struct Token {
    token: String,
}

impl ToAuth for Token {
    fn to_auth(&self) -> Authenticatable {
        Authenticatable::Token { token: &self.token }
    }
}

#[derive(Deserialize)]
pub struct TokenWithPassword {
    token: String,
    password: String,
}

impl ToAuth for TokenWithPassword {
    fn to_auth(&self) -> Authenticatable {
        Authenticatable::TokenAndPass {
            token: &self.token,
            password: &self.password,
        }
    }
}
