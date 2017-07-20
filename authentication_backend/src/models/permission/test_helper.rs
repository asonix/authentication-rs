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
use diesel;
use diesel::prelude::*;
use CONFIG;
use test_helper::*;
use super::Permission;
use super::new_permission::NewPermission;

pub fn with_permission<T>(test: T) -> ()
where
    T: FnOnce(Permission) -> () + panic::UnwindSafe,
{
    let new_permission =
        NewPermission::new(&generate_string()).expect("Failed to create New Permission");
    let permission = new_permission.save().expect("Failed to save Permission");

    let p_id = permission.id();
    let result = panic::catch_unwind(|| test(permission));
    teardown(p_id);
    result.unwrap();
}

pub fn teardown(p_id: i32) -> () {
    use schema::permissions::dsl::*;

    let _ = diesel::delete(permissions.filter(id.eq(p_id))).execute(CONFIG.db().unwrap().conn());
}

pub fn teardown_by_name(p_name: &str) -> () {
    use schema::permissions::dsl::*;

    let _ = diesel::delete(permissions.filter(name.eq(p_name)))
        .execute(CONFIG.db().unwrap().conn());
}
