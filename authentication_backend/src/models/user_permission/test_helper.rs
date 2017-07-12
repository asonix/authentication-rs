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
use models::user::User;
use models::user::test_helper::with_user;
use models::permission::Permission;
use models::permission::test_helper::with_permission;
use models::user_permission::UserPermission;
use models::user_permission::new_user_permission::NewUserPermission;

pub fn with_user_permission<T>(test: T) -> ()
where
    T: FnOnce(User, Permission, UserPermission) -> () + panic::UnwindSafe,
{
    with_user(|user| {
        with_permission(|permission| {
            let user_permission = NewUserPermission::new(&user, &permission).save().expect(
                "Failed to save NewUserPermission",
            );

            let up_id = user_permission.id;
            let result = panic::catch_unwind(|| test(user, permission, user_permission));
            teardown(up_id);
            result.unwrap();
        });
    });
}

pub fn teardown(up_id: i32) -> () {
    use schema::user_permissions::dsl::*;

    let _ = diesel::delete(user_permissions.filter(id.eq(up_id)))
        .execute(CONFIG.db().unwrap().conn());
}
