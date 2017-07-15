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

use diesel;
use diesel::prelude::*;
use CONFIG;
use std::panic;
use test_helper::*;
use error::Result;
use super::{User, NewUser};
use authenticatable::Authenticatable;

pub fn teardown(u_id: i32) -> () {
    use schema::users::dsl::{users, id};

    let _ = diesel::delete(users.filter(id.eq(u_id))).execute(CONFIG.db().unwrap().conn());
}

pub fn with_new_user<T>(test: T) -> ()
where
    T: FnOnce(NewUser) -> () + panic::UnwindSafe,
{
    let new_user = generate_new_user().expect("Failed to create NewUser for save test");
    panic::catch_unwind(|| test(new_user)).unwrap();
}

pub fn with_user<T>(test: T) -> ()
where
    T: FnOnce(User) -> () + panic::UnwindSafe,
{
    with_new_user(|new_user| {
        let user = new_user.save().expect(
            "Failed to create User for with_user",
        );

        let u_id = user.id();
        let result = panic::catch_unwind(|| test(user));
        teardown(u_id);
        result.unwrap();
    });
}

pub fn generate_new_user() -> Result<NewUser> {
    let auth = Authenticatable::UserAndPass {
        username: &generate_string(),
        password: test_password(),
    };

    NewUser::new(&auth)
}
