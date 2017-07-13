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

    pub fn authenticate(token: &str) -> Result<(i32, String)> {
        let claims = Claims::authenticate(token)?;

        Ok((claims.user_id(), claims.username().to_owned()))
    }

    pub fn renew(token: &str) -> Result<Self> {
        let claims = Claims::renew(token)?;

        Webtoken::create(claims.user_id(), claims.username())
    }

    pub fn user_token(&self) -> &str {
        &self.user_token
    }

    pub fn renewal_token(&self) -> &str {
        &self.renewal_token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_creates_webtoken() {
        let result = Webtoken::create(1, "some name");

        assert!(result.is_ok(), "Failed to create webtoken");
    }

    #[test]
    fn full_authentication_cycle_works() {
        let user_id = 1;
        let username = "some name";
        let webtoken = Webtoken::create(user_id, username).expect("Failed to create webtoken");

        let result = Webtoken::authenticate(webtoken.user_token());

        assert!(result.is_ok(), "Failed to get claims from User Token");

        let (result_id, result_name) = result.unwrap();

        assert_eq!(user_id, result_id, "User from Token has bad ID");
        assert_eq!(username, result_name, "User from Token has bad username");
    }

    #[test]
    fn full_renewal_cycle_works() {
        let user_id = 1;
        let username = "some name";
        let webtoken = Webtoken::create(user_id, username).expect("Failed to create webtoken");

        let webtoken_2 = Webtoken::renew(webtoken.renewal_token());

        assert!(webtoken_2.is_ok(), "Failed to renew webtoken");

        let webtoken_2 = webtoken_2.unwrap();

        let result = Webtoken::authenticate(webtoken_2.user_token());

        assert!(result.is_ok(), "Failed to get claims from User Token");

        let (result_id, result_name) = result.unwrap();

        assert_eq!(user_id, result_id, "User from Token has bad ID");
        assert_eq!(username, result_name, "User from Token has bad username");
    }
}
