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
use models::user::User;
use models::permission::Permission;
use self::new_user_permission::NewUserPermission;

mod new_user_permission;

#[cfg(test)]
pub mod test_helper;


#[derive(Debug, PartialEq, Queryable, Identifiable, Associations)]
#[belongs_to(User)]
#[belongs_to(Permission)]
pub struct UserPermission {
    id: i32,
    user_id: i32,
    permission_id: i32,
}

impl UserPermission {
    pub fn create(user: &User, permission: &Permission) -> Result<Self> {
        let new_user_permission = NewUserPermission::new(user, permission);

        new_user_permission.save()
    }

    pub fn has_permission(user: &User, permission: &Permission) -> bool {
        use schema::user_permissions::dsl::{user_permissions, user_id, permission_id};

        let db = match CONFIG.db() {
            Ok(db) => db,
            _ => return false,
        };

        let user_permission = user_permissions
            .filter(user_id.eq(user.id()))
            .filter(permission_id.eq(permission.id()))
            .first::<UserPermission>(db.conn());

        match user_permission {
            Ok(_permission) => true,
            _ => false,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use models::user::test_helper::with_user;
    use models::permission::Permission;
    use models::permission::test_helper::with_permission;
    use models::user_permission::test_helper::with_user_permission;

    #[test]
    fn new_user_is_not_admin() {
        with_user(|user| {
            let admin = Permission::find("admin").unwrap();

            let result = UserPermission::has_permission(&user, &admin);

            assert!(!result, "New User is Admin");
        });
    }

    #[test]
    fn can_make_user_admin() {
        with_user(|user| {
            let admin = Permission::find("admin").unwrap();

            let _ = UserPermission::create(&user, &admin).unwrap();

            let result = UserPermission::has_permission(&user, &admin);

            assert!(result, "User can become admin");
        });
    }

    #[test]
    fn get_permissions_gets_permissions() {
        with_user_permission(|user, permission, _user_permission| {
            let result = UserPermission::get_permissions(&user);

            assert!(result.is_ok(), "Failed to get Permissions for User");

            let permissions = result.unwrap();

            assert_eq!(
                permissions,
                vec![permission],
                "Retrieved permissions not accurate"
            );
        });
    }

    #[test]
    fn get_users_gets_users() {
        with_user_permission(|user, permission, _user_permission| {
            let result = UserPermission::get_users(&permission);

            assert!(result.is_ok(), "Failed to get Users with Permission");

            let users = result.unwrap();

            assert_eq!(users, vec![user], "Retrieved users not accurate");
        });
    }

    #[test]
    fn create_creates_user_permission() {
        with_user(|user| {
            with_permission(|permission| {
                let result = UserPermission::create(&user, &permission);

                assert!(result.is_ok(), "Failed to create UserPermission");
            });
        });
    }
}
