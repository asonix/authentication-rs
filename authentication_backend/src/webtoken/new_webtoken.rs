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

use jwt::{Header, Algorithm};
use CONFIG;
use error::Result;
use super::claims::Claims;
use super::Webtoken;

pub struct NewWebtoken {
    user_claims: Claims,
    renewal_claims: Claims,
}

impl NewWebtoken {
    pub fn new(user_id: i32, username: &str) -> Self {
        NewWebtoken {
            user_claims: Claims::new(user_id, username, "user", 2),
            renewal_claims: Claims::new(user_id, username, "renewal", 7),
        }
    }

    pub fn to_token(&self) -> Result<Webtoken> {
        let mut header = Header::default();
        header.alg = Algorithm::RS512;

        let secret = CONFIG.jwt_secret();

        Ok(Webtoken {
            user_token: secret.encode(&header, &self.user_claims)?,
            renewal_token: secret.encode(&header, &self.renewal_claims)?,
        })
    }
}

#[cfg(test)]
mod tests {}
