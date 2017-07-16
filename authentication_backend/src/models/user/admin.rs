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

use CONFIG;
use error::{Error, Result};
use models::{User, UserPermission, Permission, VerificationCode};
use models::user::UserTrait;

pub struct Admin<'a> {
    id: i32,
    username: &'a str,
    verified: bool,
}

impl<'a> UserTrait for Admin<'a> {
    fn id(&self) -> i32 {
        self.id
    }

    fn username(&self) -> &str {
        self.username
    }

    fn is_verified(&self) -> bool {
        self.verified
    }
}

impl<'a> Admin<'a> {
    pub fn from_user(user: &User) -> Result<Admin> {
        if user.has_permission("admin") {
            Ok(Admin {
                id: UserTrait::id(user),
                username: user.username(),
                verified: user.is_verified(),
            })
        } else {
            Err(Error::PermissionError)
        }
    }

    pub fn give_permission(&self, target: &User, permission: &str) -> Result<()> {
        let permission = Permission::find(permission)?;

        let _ = UserPermission::create(target, &permission)?;

        Ok(())
    }

    pub fn revoke_permission(&self, target: &User, permission: &str) -> Result<()> {
        let permission = Permission::find(permission)?;

        UserPermission::delete(target, &permission)
    }

    pub fn create_permission(&self, permission: &str) -> Result<Permission> {
        Permission::create(permission)
    }

    pub fn delete_permission(&self, permission: &str) -> Result<()> {
        Permission::delete(permission)
    }

    pub fn verify_user(&self, username: &str) -> Result<()> {
        let mut user = User::find_by_name(username)?;

        let db = CONFIG.db()?;

        if !user.verify(&db) {
            return Err(Error::UserNotVerifiedError);
        }

        let _ = VerificationCode::delete_by_user_id(user.id())?;

        Ok(())
    }

    pub fn delete_user(&self, uname: &str) -> Result<()> {
        use diesel;
        use diesel::prelude::*;
        use schema::users::dsl::*;

        let db = CONFIG.db()?;

        diesel::delete(users.filter(username.eq(uname))).execute(
            db.conn(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::user::test_helper::with_user;

    #[test]
    fn admin_can_give_permissions_to_non_admins() {
        use models::Permission;
        use models::UserPermission;

        with_user(|user| {
            let admin_permission =
                Permission::find("admin").expect("Failed to find admin permission");

            let _ = UserPermission::create(&user, &admin_permission).expect(
                "Failed to make test admin user_permission",
            );

            let admin = Admin::from_user(user).expect(
                "Failed to get Admin from User with 'admin' permission",
            );

            with_user(|user| {
                let result = admin.give_permission(&user, "admin");

                assert!(result.is_ok(), "Admin failed to give user new permission");
            });
        });
    }

    #[test]
    fn admin_cannot_give_nonexistant_permission() {
        use models::permission::Permission;
        use models::user_permission::UserPermission;

        with_user(|user| {
            let admin_permission =
                Permission::find("admin").expect("Failed to find admin permission");

            let _ = UserPermission::create(&user, &admin_permission).expect(
                "Failed to make test admin user_permission",
            );

            let admin = Admin::from_user(&user).expect(
                "Failed to get Admin from User with 'admin' permission",
            );

            with_user(|user| {
                let result = admin.give_permission(&user, "this is not a permission");

                assert!(
                    !result.is_ok(),
                    "Admin gave a user a nonexistant permission"
                );
            });
        });
    }
}
