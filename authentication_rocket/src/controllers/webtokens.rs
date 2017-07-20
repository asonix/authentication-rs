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

use authentication_backend::Webtoken;
use routes::Response;
use auth_response::AuthResponse;

pub fn renew(renewal_token: &str) -> Response {
    let webtoken = Webtoken::renew(renewal_token)?;

    Ok(AuthResponse::new("Renewed", webtoken))
}

#[cfg(test)]
mod tests {
    use super::*;
    use authentication_backend::webtoken_test_helper::with_token;

    #[test]
    fn renew_renews_tokens() {
        with_token("renewal", |token| {
            let result = renew(token);

            assert!(result.is_ok(), "Failed to renew token");
        });
    }

    #[test]
    fn renew_fails_with_bad_token() {
        with_token("invalid", |token| {
            let result = renew(token);

            assert!(!result.is_ok(), "Renewed bad token");
        });
    }
}
