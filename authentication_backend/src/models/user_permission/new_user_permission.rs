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
use error::Result;
use schema::user_permissions;
use models::{Permission, UserPermission};
use models::user::UserTrait;

#[derive(Debug, Insertable)]
#[table_name = "user_permissions"]
pub struct NewUserPermission {
    user_id: i32,
    permission_id: i32,
}

impl NewUserPermission {
    pub fn new<T>(user: &T, permission: &Permission) -> Self
    where
        T: UserTrait,
    {
        NewUserPermission {
            user_id: UserTrait::id(user),
            permission_id: permission.id(),
        }
    }

    pub fn save(&self) -> Result<UserPermission> {
        use schema::user_permissions::dsl::*;

        let db = CONFIG.db()?;

        Ok(diesel::insert(self).into(user_permissions).get_result(
            db.conn(),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::user::test_helper::with_user;
    use models::permission::test_helper::with_permission;

    #[test]
    fn save_saves_new_user_permission() {
        with_user(|user| {
            with_permission(|permission| {
                let result = NewUserPermission::new(&user, &permission).save();

                assert!(result.is_ok(), "Failed to save NewUserPermission");
            });
        });
    }
}
