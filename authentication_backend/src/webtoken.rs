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

use CONFIG;
use jwt::{Header, Algorithm, Validation};
use error::Result;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    user_id: i32,
    username: String,
}

impl Claims {
    pub fn new(user_id: i32, username: &str) -> Self {
        Claims {
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

    pub fn to_token(&self) -> Result<String> {
        let mut header = Header::default();
        header.alg = Algorithm::RS512;

        CONFIG.jwt_secret().encode(&header, &self)
    }

    pub fn from_token(token: &str) -> Result<Self> {
        let validation = Validation {leeway: 1000*30, algorithms: Some(vec![Algorithm::RS512]), ..Default::default()};

        CONFIG.jwt_secret().decode(token, &validation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_token_creates_token() {
        let claims = Claims::new(1, "hello");

        let result = claims.to_token();

        assert!(result.is_ok(), "Failed to create token from claims");
    }

    #[test]
    fn from_token_creates_claims() {
        let claims = Claims::new(1, "hello");

        let token = claims.to_token().expect(
            "Failed to create token from claims",
        );
        let result = Claims::from_token(&token);

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
            "Token returns different user_id from start"
        );
    }
}
