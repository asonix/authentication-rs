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

use std::convert::From;
use authentication_backend::Webtoken;
use authentication_backend::User;
use authentication_backend::Permission;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ResponseBody {
    User { id: i32, username: String },
    Webtoken {
        user_token: String,
        renewal_token: String,
    },
    Permission { id: i32, name: String },
    NoData,
}

impl From<Permission> for ResponseBody {
    fn from(permission: Permission) -> Self {
        ResponseBody::Permission {
            id: permission.id(),
            name: permission.name().to_owned(),
        }
    }
}

impl<T: Into<ResponseBody>> From<Option<T>> for ResponseBody {
    fn from(option: Option<T>) -> Self {
        if let Some(value) = option {
            value.into()
        } else {
            ResponseBody::NoData
        }
    }
}

impl From<Webtoken> for ResponseBody {
    fn from(webtoken: Webtoken) -> Self {
        ResponseBody::Webtoken {
            user_token: webtoken.user_token().to_owned(),
            renewal_token: webtoken.renewal_token().to_owned(),
        }
    }
}

impl From<User> for ResponseBody {
    fn from(user: User) -> Self {
        ResponseBody::User {
            id: user.id(),
            username: user.username().to_owned(),
        }
    }
}

impl ResponseBody {
    pub fn is_empty(&self) -> bool {
        match *self {
            ResponseBody::NoData => true,
            _ => false,
        }
    }
}
