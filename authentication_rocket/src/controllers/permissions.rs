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

use authentication_backend::{ToAuth, Admin, User};
use routes::Response;
use auth_response::AuthResponse;

pub fn create<T>(permission: &str, auth: &T) -> Response
where
    T: ToAuth,
{
    let user = User::authenticate(auth)?;
    let admin = Admin::from_authenticated(user)?;

    let permission = admin.create_permission(permission)?;

    Ok(AuthResponse::new("Permission created", permission))
}

pub fn delete<T>(permission: &str, auth: &T) -> Response
where
    T: ToAuth,
{
    let user = User::authenticate(auth)?;
    let admin = Admin::from_authenticated(user)?;

    admin.delete_permission(permission)?;

    Ok(AuthResponse::empty("Permission deleted"))
}

#[cfg(test)]
mod tests {
    use std::panic;
    use super::*;
    use authentication_backend::{Authenticatable, UserTrait};
    use authentication_backend::permission_test_helper::{with_permission, teardown_by_name};
    use authentication_backend::user_test_helper::{with_admin, with_user};
    use authentication_backend::test_helper::{generate_string, test_password};

    #[test]
    fn create_creates_permission() {
        with_admin(|admin| {
            test_wrapper(|permission| {
                let auth = Authenticatable::UserAndPass {
                    username: admin.username(),
                    password: test_password(),
                };

                let result = create(permission, &auth);

                assert!(result.is_ok(), "Failed to create permission");
            });
        });
    }

    #[test]
    fn user_cannot_create_permission() {
        with_user(|user| {
            test_wrapper(|permission| {
                let auth = Authenticatable::UserAndPass {
                    username: user.username(),
                    password: test_password(),
                };

                let result = create(permission, &auth);

                assert!(!result.is_ok(), "Failed to create permission");
            });
        });
    }

    #[test]
    fn delete_deletes_permission() {
        with_admin(|admin| {
            with_permission(|permission| {
                let auth = Authenticatable::UserAndPass {
                    username: admin.username(),
                    password: test_password(),
                };

                let result = delete(permission.name(), &auth);

                assert!(result.is_ok(), "Failed to delete permission");
            });
        });
    }

    #[test]
    fn user_cannot_delete_permission() {
        with_user(|user| {
            with_permission(|permission| {
                let auth = Authenticatable::UserAndPass {
                    username: user.username(),
                    password: test_password(),
                };

                let result = delete(permission.name(), &auth);

                assert!(!result.is_ok(), "Failed to delete permission");
            });
        });
    }

    fn test_wrapper<T>(test: T)
    where
        T: FnOnce(&str) -> () + panic::UnwindSafe,
    {
        let permission = generate_string();
        let result = panic::catch_unwind(|| test(&permission));
        teardown_by_name(&permission);
        result.unwrap();
    }
}
