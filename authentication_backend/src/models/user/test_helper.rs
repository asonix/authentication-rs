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
use super::{UserTrait, User, NewUser, Admin, Authenticated, AuthenticatedThisSession};
use models::{Permission, UserPermission};
use authenticatable::Authenticatable;

pub fn teardown(u_id: i32) -> () {
    use schema::users::dsl::{users, id};

    let _ = diesel::delete(users.filter(id.eq(u_id))).execute(CONFIG.db().unwrap().conn());
}

pub fn teardown_by_name(u_name: &str) -> () {
    use schema::users::dsl::{users, username};

    let _ = diesel::delete(users.filter(username.eq(u_name))).execute(CONFIG.db().unwrap().conn());
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

        let u_id = UserTrait::id(&user);
        let result = panic::catch_unwind(|| test(user));
        teardown(u_id);
        result.unwrap();
    });
}

pub fn with_admin<T>(test: T) -> ()
where
    T: FnOnce(Admin) -> () + panic::UnwindSafe,
{
    with_user(|user| {
        let admin_permission = Permission::find("admin").expect("Failed to find admin permission");

        let _ = UserPermission::create(&user, &admin_permission).expect(
            "Failed to make test admin user_permission",
        );

        let auth = Authenticatable::UserAndPass {
            username: user.username(),
            password: test_password(),
        };

        let auth = User::authenticate(&auth).expect("Failed to authenticate");

        let admin = Admin::from_authenticated(auth).expect(
            "Failed to get Admin from User with 'admin' permission",
        );

        panic::catch_unwind(|| test(admin)).unwrap();
    });
}

pub fn with_authenticated<T>(test: T) -> ()
where
    T: FnOnce(Authenticated) -> () + panic::UnwindSafe,
{
    with_user(|user| {
        let auth = Authenticatable::UserAndPass {
            username: user.username(),
            password: test_password(),
        };

        let auth = User::authenticate(&auth).expect("Failed to authenticate");

        panic::catch_unwind(|| test(auth)).unwrap();
    });
}

pub fn with_auth_session<T>(test: T) -> ()
where
    T: FnOnce(AuthenticatedThisSession) -> () + panic::UnwindSafe,
{
    with_user(|user| {
        let auth = Authenticatable::UserAndPass {
            username: user.username(),
            password: test_password(),
        };

        let auth = User::authenticate_session(&auth).expect("Failed to authenticate");

        panic::catch_unwind(|| test(auth)).unwrap();
    });
}

pub fn generate_new_user() -> Result<NewUser> {
    let auth = Authenticatable::UserAndPass {
        username: &generate_string(),
        password: test_password(),
    };

    NewUser::new(&auth)
}
