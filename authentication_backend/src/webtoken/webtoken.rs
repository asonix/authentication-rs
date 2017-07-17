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
use models::UserTrait;
use super::claims::Claims;
use super::new_webtoken::NewWebtoken;

pub struct Webtoken {
    user_token: String,
    renewal_token: String,
}

impl Webtoken {
    pub fn new(user: &str, renewal: &str) -> Self {
        Webtoken {
            user_token: user.to_owned(),
            renewal_token: renewal.to_owned(),
        }
    }

    pub fn create<T>(user: &T) -> Result<Self>
    where
        T: UserTrait,
    {
        let new_webtoken = NewWebtoken::new(user);

        let webtoken = new_webtoken.to_token()?;

        Ok(webtoken)
    }

    pub fn authenticate(token: &str) -> Result<(i32, String, bool, bool)> {
        let claims = Claims::authenticate(token)?;

        Ok((
            claims.id(),
            claims.username().to_owned(),
            claims.is_verified(),
            claims.is_admin(),
        ))
    }

    pub fn renew(token: &str) -> Result<Self> {
        let claims = Claims::renew(token)?;

        Webtoken::create(&claims)
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
    use models::test_helper::with_authenticated;

    #[test]
    fn create_creates_webtoken() {
        with_authenticated(|authenticated| {
            let result = Webtoken::create(&authenticated);

            assert!(result.is_ok(), "Failed to create webtoken");
        });
    }

    #[test]
    fn full_authentication_cycle_works() {
        with_authenticated(|authenticated| {
            let webtoken = Webtoken::create(&authenticated).expect("Failed to create webtoken");

            let result = Webtoken::authenticate(webtoken.user_token());

            assert!(result.is_ok(), "Failed to get claims from User Token");

            let (result_id, result_name, result_verified, _admin) = result.unwrap();

            assert_eq!(authenticated.id(), result_id, "User from Token has bad ID");
            assert_eq!(
                authenticated.username(),
                result_name,
                "User from Token has bad username"
            );
            assert_eq!(
                authenticated.is_verified(),
                result_verified,
                "User from Token has bad Verification status"
            );
        });
    }

    #[test]
    fn full_renewal_cycle_works() {
        with_authenticated(|authenticated| {
            let webtoken = Webtoken::create(&authenticated).expect("Failed to create webtoken");

            let webtoken_2 = Webtoken::renew(webtoken.renewal_token());

            assert!(webtoken_2.is_ok(), "Failed to renew webtoken");

            let webtoken_2 = webtoken_2.unwrap();

            let result = Webtoken::authenticate(webtoken_2.user_token());

            assert!(result.is_ok(), "Failed to get claims from User Token");

            let (result_id, result_name, result_verified, _admin) = result.unwrap();

            assert_eq!(authenticated.id(), result_id, "User from Token has bad ID");
            assert_eq!(
                authenticated.username(),
                result_name,
                "User from Token has bad username"
            );
            assert_eq!(
                authenticated.is_verified(),
                result_verified,
                "User from Token has bad username"
            );
        });
    }
}
