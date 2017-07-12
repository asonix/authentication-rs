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
mod tests {
    use super::*;
    use std::panic;
    use models::user::{NewUser, User, Authenticatable};
    use models::permission::{NewPermission, Permission};

    #[test]
    fn create_creates_user_permission() {
        with_user(|user| {
            with_permission(|permission| {
                let result = UserPermission::create(&user, &permission);

                assert!(result.is_ok(), "Failed to create UserPermission");
            });
        });
    }

    fn with_permission<T>(test: T) -> ()
    where
        T: FnOnce(Permission) -> () + panic::UnwindSafe,
    {
        let new_permission =
            NewPermission::new(&generate_string()).expect("Failed to create New Permission");
        let permission = new_permission.save().expect("Failed to save Permission");

        let p_id = permission.id();
        let result = panic::catch_unwind(|| test(permission));
        permission_teardown(p_id);
        result.unwrap();
    }

    fn with_user<T>(test: T) -> ()
    where
        T: FnOnce(User) -> () + panic::UnwindSafe,
    {
        let auth = Authenticatable::UserAndPass {
            username: &generate_string(),
            password: &test_password(),
        };
        let new_user = NewUser::new(&auth).expect("Failed to create New User");
        let user = new_user.save().expect("Failed to save User");

        let u_id = user.id();
        let result = panic::catch_unwind(|| test(user));
        user_teardown(u_id);
        result.unwrap();
    }

    fn permission_teardown(p_id: i32) -> () {
        use schema::permissions::dsl::*;
        let _ =
            diesel::delete(permissions.filter(id.eq(p_id))).execute(CONFIG.db().unwrap().conn());
    }

    fn user_teardown(u_id: i32) -> () {
        use schema::users::dsl::*;
        let _ = diesel::delete(users.filter(id.eq(u_id))).execute(CONFIG.db().unwrap().conn());
    }

    fn generate_string() -> String {
        use rand::Rng;
        use rand::OsRng;

        OsRng::new().unwrap().gen_ascii_chars().take(10).collect()
    }

    fn test_password() -> String {
        "Passw0rd$.".to_string()
    }
}
