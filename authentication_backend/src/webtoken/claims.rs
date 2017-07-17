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

use jwt::{Algorithm, Validation};
use chrono::{Utc, Duration};
use CONFIG;
use models::UserTrait;
use error::Result;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    iss: String,
    sub: String,
    iat: i64,
    exp: i64,
    user_id: i32,
    username: String,
    verified: bool,
}

impl UserTrait for Claims {
    fn id(&self) -> i32 {
        self.user_id
    }

    fn username(&self) -> &str {
        &self.username
    }

    fn is_verified(&self) -> bool {
        self.verified
    }
}

impl Claims {
    pub fn new<T>(user: &T, subject: &str, days: i64) -> Self
    where
        T: UserTrait,
    {
        let issued_at = Utc::now();
        let expiration = issued_at + Duration::days(days);

        Claims {
            iss: "authentication".to_owned(),
            sub: subject.to_owned(),
            iat: issued_at.timestamp(),
            exp: expiration.timestamp(),
            user_id: user.id(),
            username: user.username().to_owned(),
            verified: user.is_verified(),
        }
    }

    pub fn authenticate(token: &str) -> Result<Self> {
        let validation = Validation {
            leeway: 1000 * 30,
            algorithms: Some(vec![Algorithm::RS512]),
            iss: Some("authentication".to_owned()),
            sub: Some("user".to_owned()),
            ..Default::default()
        };

        CONFIG.jwt_secret().decode(token, &validation)
    }

    pub fn renew(token: &str) -> Result<Self> {
        let validation = Validation {
            leeway: 1000 * 30,
            algorithms: Some(vec![Algorithm::RS512]),
            iss: Some("authentication".to_owned()),
            sub: Some("renewal".to_owned()),
            ..Default::default()
        };

        CONFIG.jwt_secret().decode(token, &validation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use webtoken::test_helper::{with_claims, with_token};
    use jwt::Header;

    #[test]
    fn renew_creates_claims() {
        with_claims("renewal", |claims| {
            let mut header = Header::default();
            header.alg = Algorithm::RS512;

            let token: String = CONFIG.jwt_secret().encode(&header, &claims).expect(
                "Failed to create token from claims",
            );

            let result = Claims::renew(&token);

            assert!(result.is_ok(), "Failed to get claims from token");

            let result = result.unwrap();

            assert_eq!(
                result.id(),
                claims.id(),
                "Token returns different user_id from start"
            );
            assert_eq!(
                result.username(),
                claims.username(),
                "Token returns different username from start"
            );
        });
    }

    #[test]
    fn authenticate_creates_claims() {
        with_claims("user", |claims| {
            let mut header = Header::default();
            header.alg = Algorithm::RS512;

            let token: String = CONFIG.jwt_secret().encode(&header, &claims).expect(
                "Failed to create token from claims",
            );

            let result = Claims::authenticate(&token);

            assert!(result.is_ok(), "Failed to get claims from token");

            let result = result.unwrap();

            assert_eq!(
                result.id(),
                claims.id(),
                "Token returns different user_id from start"
            );
            assert_eq!(
                result.username(),
                claims.username(),
                "Token returns different username from start"
            );
        });
    }

    #[test]
    fn renew_fails_with_user_token() {
        with_token("user", |token| {
            let result = Claims::renew(token);

            assert!(!result.is_ok(), "Validated User token as Renewal token");
        });
    }

    #[test]
    fn authenticate_fails_with_renewal_token() {
        with_token("renewal", |token| {
            let result = Claims::authenticate(token);

            assert!(!result.is_ok(), "Validated User token as Renewal token");
        });
    }

    #[test]
    fn authenticate_fails_with_fake_token() {
        let result = Claims::authenticate("This is not a webtoken");

        assert!(!result.is_ok(), "Created claims from fake webtoken");
    }

    #[test]
    fn renew_fails_with_fake_token() {
        let result = Claims::renew("This is not a webtoken");

        assert!(!result.is_ok(), "Created claims from fake webtoken");
    }
}
