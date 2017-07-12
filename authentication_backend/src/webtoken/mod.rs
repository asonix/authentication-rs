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

use error::Result;
use self::claims::Claims;
use self::new_webtoken::NewWebtoken;

mod claims;
mod new_webtoken;

pub struct Webtoken {
    user_token: String,
    renewal_token: String,
}

impl Webtoken {
    pub fn create(user_id: i32, username: &str) -> Result<Self> {
        let new_webtoken = NewWebtoken::new(user_id, username);

        let webtoken = new_webtoken.to_token()?;

        Ok(webtoken)
    }

    pub fn from_user_token(token: &str) -> Result<(i32, String)> {
        let claims = Claims::from_user_token(token)?;

        Ok((claims.user_id(), claims.username().to_owned()))
    }

    pub fn from_renewal_token(token: &str) -> Result<Self> {
        let claims = Claims::from_renewal_token(token)?;

        Webtoken::create(claims.user_id(), claims.username())
    }

    pub fn user_token(&self) -> &str {
        &self.user_token
    }

    pub fn renewal_token(&self) -> &str {
        &self.renewal_token
    }
}
