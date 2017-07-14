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
pub struct CreatePermission {
    authorizing_user: UserToken,
    permission: String,
}

impl CreatePermission {
    pub fn permission(&self) -> &str {
        &self.permission
    }
}

impl ToAuth for CreatePermission {
    fn to_auth(&self) -> Authenticatable {
        self.authorizing_user.to_auth()
    }
}

#[derive(Deserialize)]
pub struct GivePermission {
    authorizing_user: UserToken,
    target_user: String,
    permission: String,
}

impl GivePermission {
    pub fn permission(&self) -> &str {
        &self.permission
    }

    pub fn target_user(&self) -> &str {
        &self.target_user
    }
}

impl ToAuth for GivePermission {
    fn to_auth(&self) -> Authenticatable {
        self.authorizing_user.to_auth()
    }
}

#[derive(Deserialize)]
pub struct CreateUser {
    username: String,
    password: String,
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
pub struct UserToken {
    user_token: String,
}

impl ToAuth for UserToken {
    fn to_auth(&self) -> Authenticatable {
        Authenticatable::UserToken { user_token: &self.user_token }
    }
}

#[derive(Deserialize)]
pub struct RenewalToken {
    pub renewal_token: String,
}

#[derive(Deserialize)]
pub struct UserTokenWithPassword {
    user_token: String,
    password: String,
}

impl ToAuth for UserTokenWithPassword {
    fn to_auth(&self) -> Authenticatable {
        Authenticatable::UserTokenAndPass {
            user_token: &self.user_token,
            password: &self.password,
        }
    }
}
