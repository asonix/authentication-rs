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

use error::{Error, Result};
use models::{User, UserPermission, Permission};

pub struct Admin<'a> {
    id: i32,
    username: &'a str,
}

impl<'a> Admin<'a> {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn username(&self) -> &'a str {
        self.username
    }

    pub fn from_user(user: &User) -> Result<Admin> {
        if user.has_permission("admin") {
            Ok(Admin {
                id: user.id(),
                username: user.username(),
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

    pub fn create_permission(&self, permission: &str) -> Result<Permission> {
        Permission::create(permission)
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
