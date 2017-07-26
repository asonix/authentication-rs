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

pub enum Authenticatable<'a> {
    UserAndPass {
        username: &'a str,
        password: &'a str,
    },
    UserToken { user_token: &'a str },
    UserTokenAndPass {
        user_token: &'a str,
        password: &'a str,
    },
}

pub trait ToAuth {
    fn to_auth(&self) -> Authenticatable;
}

impl<'a> ToAuth for Authenticatable<'a> {
    fn to_auth(&self) -> Authenticatable {
        match *self {
            Authenticatable::UserAndPass {
                username: u,
                password: p,
            } => Authenticatable::UserAndPass {
                username: u,
                password: p,
            },
            Authenticatable::UserToken { user_token: u } => Authenticatable::UserToken {
                user_token: u,
            },
            Authenticatable::UserTokenAndPass {
                user_token: u,
                password: p,
            } => Authenticatable::UserTokenAndPass {
                user_token: u,
                password: p,
            },
        }
    }
}
