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
use models::user::{UserTrait, Authenticated};

pub struct Admin {
    id: i32,
    username: String,
    verified: bool,
}

impl UserTrait for Admin {
    fn id(&self) -> i32 {
        self.id
    }

    fn username(&self) -> &str {
        &self.username
    }

    fn is_verified(&self) -> bool {
        self.verified
    }
}

impl Admin {
    pub fn from_authenticated<T>(auth: T) -> Result<Admin>
    where
        T: Into<Authenticated>,
    {
        use models::{UserPermission, Permission};

        let permission = Permission::find("admin")?;

        let auth: Authenticated = auth.into();

        let has_permission: bool = UserPermission::has_permission(&auth, &permission);

        if has_permission {
            Ok(Admin {
                id: UserTrait::id(&auth),
                username: auth.username().to_owned(),
                verified: auth.is_verified(),
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
    use test_helper::*;
    use models::user::test_helper::with_user;
    use authenticatable::Authenticatable;

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

            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: test_password(),
            };

            let auth = User::authenticate(&auth).expect("Failed to authenticate");

            let admin = Admin::from_authenticated(auth).expect(
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

            let auth = Authenticatable::UserAndPass {
                username: user.username(),
                password: test_password(),
            };

            let auth = User::authenticate(&auth).expect("Failed to authenticate");

            let admin = Admin::from_authenticated(auth).expect(
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
