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

use std::panic;
use jwt::{Header, Algorithm};
use CONFIG;
use super::claims::Claims;
use models::test_helper::with_authenticated;

pub fn with_claims<T>(sub: &str, test: T) -> ()
where
    T: FnOnce(Claims) -> () + panic::UnwindSafe,
{
    with_authenticated(|authenticated| {
        let claims = Claims::new(&authenticated, sub, 2);

        panic::catch_unwind(|| test(claims)).unwrap();
    });
}

pub fn with_token<T>(sub: &str, test: T) -> ()
where
    T: FnOnce(&str) -> () + panic::UnwindSafe,
{
    with_claims(sub, |claims| {
        let mut header = Header::default();
        header.alg = Algorithm::RS512;

        let token: String = CONFIG.jwt_secret().encode(&header, &claims).expect(
            "Failed to create token from claims",
        );

        panic::catch_unwind(|| test(&token)).unwrap();
    });
}
