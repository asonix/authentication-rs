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
use schema::user_permissions;
use error::Result;
use models::user::User;
use models::permission::Permission;
use CONFIG;

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(User)]
#[belongs_to(Permission)]
pub struct UserPermission {
    id: i32,
    user_id: i32,
    permission_id: i32,
}

#[derive(Insertable)]
#[table_name = "user_permissions"]
pub struct NewUserPermission {
    user_id: i32,
    permission_id: i32,
}

impl UserPermission {
    pub fn create(user: &User, permission: &Permission) -> Result<Self> {
        let new_user_permission = NewUserPermission::new(user, permission);

        new_user_permission.save()
    }

    pub fn get_permissions(user: &User) -> Result<Vec<Permission>> {
        use schema::user_permissions::dsl::{user_permissions, user_id, permission_id};
        use schema::permissions::dsl::{id, permissions};

        let db = CONFIG.db()?;

        let results: Vec<(UserPermission, Permission)> =
            user_permissions
                .inner_join::<permissions>(permissions)
                .filter(permission_id.eq(id))
                .filter(user_id.eq(user.id()))
                .load::<(UserPermission, Permission)>(db.conn())?;

        Ok(
            results
                .into_iter()
                .map(|(_, permission)| permission)
                .collect(),
        )
    }

    pub fn get_users(permission: &Permission) -> Result<Vec<User>> {
        use schema::user_permissions::dsl::{user_permissions, user_id, permission_id};
        use schema::users::dsl::{id, users};

        let db = CONFIG.db()?;

        let results: Vec<(UserPermission, User)> = user_permissions
            .inner_join::<users>(users)
            .filter(user_id.eq(id))
            .filter(permission_id.eq(permission.id()))
            .load::<(UserPermission, User)>(db.conn())?;

        Ok(results.into_iter().map(|(_, user)| user).collect())
    }

    pub fn delete(user: &User, permission: &Permission) -> Result<()> {
        use schema::user_permissions::dsl::{user_permissions, user_id, permission_id};

        let db = CONFIG.db()?;

        diesel::delete(user_permissions.filter(user_id.eq(user.id())).filter(
            permission_id.eq(permission.id()),
        )).execute(db.conn())?;

        Ok(())
    }
}

impl NewUserPermission {
    fn new(user: &User, permission: &Permission) -> Self {
        NewUserPermission {
            user_id: user.id(),
            permission_id: permission.id(),
        }
    }

    fn save(&self) -> Result<UserPermission> {
        let db = CONFIG.db()?;

        Ok(diesel::insert(self)
            .into(user_permissions::table)
            .get_result(db.conn())?)
    }
}

#[cfg(test)]
mod tests {}
