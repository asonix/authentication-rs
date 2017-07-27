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

use authentication_backend::{Authenticatable, ToAuth};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Auth {
    UserAndPass { username: String, password: String },
    WebtokenAndPass {
        user_token: String,
        password: String,
    },
    Webtoken { user_token: String },
}

impl ToAuth for Auth {
    fn to_auth(&self) -> Authenticatable {
        match *self {
            Auth::UserAndPass {
                username: ref u,
                password: ref p,
            } => {
                Authenticatable::UserAndPass {
                    username: u,
                    password: p,
                }
            }
            Auth::WebtokenAndPass {
                user_token: ref w,
                password: ref p,
            } => {
                Authenticatable::UserTokenAndPass {
                    user_token: w,
                    password: p,
                }
            }
            Auth::Webtoken { user_token: ref w } => Authenticatable::UserToken { user_token: w },
        }
    }
}
