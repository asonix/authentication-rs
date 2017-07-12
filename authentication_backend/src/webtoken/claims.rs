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
use error::Result;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    iss: String,
    sub: String,
    iat: i64,
    exp: i64,
    user_id: i32,
    username: String,
}

impl Claims {
    pub fn new(user_id: i32, username: &str, subject: &str, days: i64) -> Self {
        let issued_at = Utc::now();
        let expiration = issued_at + Duration::days(days);

        Claims {
            iss: "authentication".to_owned(),
            sub: subject.to_owned(),
            iat: issued_at.timestamp(),
            exp: expiration.timestamp(),
            user_id: user_id,
            username: username.to_string(),
        }
    }

    pub fn user_id(&self) -> i32 {
        self.user_id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn from_user_token(token: &str) -> Result<Self> {
        let validation = Validation {
            leeway: 1000 * 30,
            algorithms: Some(vec![Algorithm::RS512]),
            iss: Some("authentication".to_owned()),
            sub: Some("user".to_owned()),
            ..Default::default()
        };

        CONFIG.jwt_secret().decode(token, &validation)
    }

    pub fn from_renewal_token(token: &str) -> Result<Self> {
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

    #[test]
    fn to_token_creates_token() {
        let claims = Claims::new(1, "hello", "user", 2);

        let result = claims.to_token();

        assert!(result.is_ok(), "Failed to create token from claims");
    }

    #[test]
    fn from_user_token_creates_claims() {
        let claims = Claims::new(1, "hello", "user", 2);

        let token = claims.to_token().expect(
            "Failed to create token from claims",
        );
        let result = Claims::from_user_token(&token);

        assert!(result.is_ok(), "Failed to get claims from token");

        let result = result.unwrap();

        assert_eq!(
            result.user_id(),
            claims.user_id(),
            "Token returns different user_id from start"
        );
        assert_eq!(
            result.username(),
            claims.username(),
            "Token returns different username from start"
        );
    }

    #[test]
    fn from_user_token_fails_with_fake_token() {
        let result = Claims::from_token("This is not a webtoken");

        assert!(!result.is_ok(), "Created claims from fake webtoken");
    }
}
